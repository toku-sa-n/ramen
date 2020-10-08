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

pub struct RingQueue<'a, T: TRB> {
    queue: &'a mut [RawTrb<T>],
    dequeue_index: TrbPtr,
}

impl<'a, T: TRB> RingQueue<'a, T> {
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
        }
    }

    pub fn addr(&self) -> VirtAddr {
        VirtAddr::new(self.queue.as_ptr() as u64)
    }
}

pub trait TRB {
    const SIZE: usize = 16;
}

pub struct Command;
impl TRB for Command {}

pub struct Event;
impl TRB for Event {}

#[repr(transparent)]
struct RawTrb<T: TRB> {
    trb: [u32; 4],
    _marker: PhantomData<T>,
}

struct TrbPtr(usize);
impl TrbPtr {
    fn new(index: usize) -> Self {
        assert!(index < NUM_OF_TRB_IN_QUEUE);
        Self(index)
    }
}
