// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
    core::{
        convert::TryFrom,
        marker::PhantomData,
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
    queue: &'a mut [Trb<T>],
    dequeue_index: usize,
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
            dequeue_index: 0,
            cycle_bit: CycleBit::new(1),
        }
    }

    pub fn addr(&self) -> VirtAddr {
        VirtAddr::new(self.queue.as_ptr() as u64)
    }
}

impl<'a> RingQueue<'a, Event> {
    pub fn dequeue(&mut self) -> Option<Trb<Event>> {
        if self.queue[self.dequeue_index].valid(self.cycle_bit) {
            let element = self.queue[self.dequeue_index];
            self.increment_dequeue_index();
            Some(element)
        } else {
            None
        }
    }

    pub fn check(&self) {
        for (i, trb) in self.queue.iter().enumerate() {
            if trb.valid(self.cycle_bit) {
                info!("TRB{} is valid.", i);
            }
        }

        info!("Check finished.");
    }

    fn increment_dequeue_index(&mut self) {
        self.dequeue_index += 1;
        if self.dequeue_index >= NUM_OF_TRB_IN_QUEUE {
            self.dequeue_index = 0;
            self.cycle_bit.toggle();
        }
    }
}

pub trait TrbType {
    const SIZE: usize = 16;
}

pub struct Command;
impl TrbType for Command {}

#[derive(Copy, Clone)]
pub struct Event;
impl TrbType for Event {}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Trb<T: TrbType> {
    trb: [u32; 4],
    _marker: PhantomData<T>,
}
impl<T: TrbType> Trb<T> {
    pub fn ty(&self) -> u32 {
        (self.trb[3] >> 10) & 0x3f
    }

    fn valid(&self, cycle_bit: CycleBit) -> bool {
        self.trb[3] & 1 == cycle_bit.0
    }
}

#[derive(Copy, Clone)]
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
