// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
    core::{
        convert::TryFrom,
        marker::PhantomData,
        ops::{Index, IndexMut},
        ptr::{self, NonNull},
        slice,
    },
    x86_64::{
        structures::paging::{FrameAllocator, Mapper, PageSize, PageTableFlags, Size4KiB},
        VirtAddr,
    },
};

// 4KB / size_of(TRB) = 256.
const NUM_OF_TRB_IN_QUEUE: usize = 256;

pub struct RingQueue<'a, T: TrbType> {
    queue: &'a mut [RawTrb<T>],
    dequeue_index: TrbPtr,
    cycle_bit: CycleBit,
}

impl<'a, T: TrbType> RingQueue<'a, T> {
    pub fn new() -> Self {
        let page = virt::search_first_unused_page().unwrap();
        let frame = FRAME_MANAGER.lock().allocate_frame().unwrap();

        unsafe {
            PML4.lock()
                .map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT,
                    &mut *FRAME_MANAGER.lock(),
                )
                .unwrap()
                .flush();
        }

        let ptr = NonNull::<u8>::new(page.start_address().as_mut_ptr()).unwrap();

        unsafe { ptr::write_bytes(ptr.as_ptr(), 0, usize::try_from(Size4KiB::SIZE).unwrap()) }

        Self {
            queue: unsafe { slice::from_raw_parts_mut(ptr.cast().as_ptr(), NUM_OF_TRB_IN_QUEUE) },
            dequeue_index: TrbPtr::new(0),
            cycle_bit: CycleBit::new(1),
        }
    }

    pub fn addr(&self) -> VirtAddr {
        VirtAddr::new(self.queue.as_ptr() as u64)
    }
}
impl<'a, T: TrbType> Index<TrbPtr> for RingQueue<'a, T> {
    type Output = RawTrb<T>;

    fn index(&self, index: TrbPtr) -> &Self::Output {
        &self.queue[index.0]
    }
}
impl<'a, T: TrbType> IndexMut<TrbPtr> for RingQueue<'a, T> {
    fn index_mut(&mut self, index: TrbPtr) -> &mut Self::Output {
        &mut self.queue[index.0]
    }
}

impl<'a> RingQueue<'a, Event> {}

pub trait TrbType {
    const SIZE: usize = 16;
}

pub struct Command;
impl TrbType for Command {}

pub struct Event;
impl TrbType for Event {}

#[repr(transparent)]
struct RawTrb<T: TrbType> {
    trb: [u32; 4],
    _marker: PhantomData<T>,
}
impl<T: TrbType> RawTrb<T> {
    fn valid(&self, cycle_bit: CycleBit) -> bool {
        self.trb[3] & 1 == cycle_bit.0
    }
}

struct TrbPtr(usize);
impl TrbPtr {
    fn new(index: usize) -> Self {
        assert!(index < NUM_OF_TRB_IN_QUEUE);
        Self(index)
    }

    fn increment(&mut self) {
        self.0 += 1;
        self.0 %= NUM_OF_TRB_IN_QUEUE;
    }
}

struct CycleBit(u32);
impl CycleBit {
    fn new(bit: u32) -> Self {
        assert!(bit == 0 || bit == 1);
        Self(bit)
    }

    fn toggle(&mut self) {
        self.0 ^= 1;
    }
}
