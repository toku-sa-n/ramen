// SPDX-License-Identifier: GPL-3.0-or-later

pub mod transfer_request_block;

use {
    crate::mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
    core::{convert::TryFrom, marker::PhantomData, ptr, slice},
    transfer_request_block::TRB,
    x86_64::{
        structures::paging::{FrameAllocator, Mapper, PageSize, PageTableFlags, Size4KiB},
        VirtAddr,
    },
};

// 4KB / size_of(TRB) = 256.
const NUM_OF_TRB_IN_QUEUE: usize = 256;

pub struct RingQueue<'a, T: TRB> {
    queue: &'a mut [T],
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

        let ptr = page.start_address().as_mut_ptr();

        unsafe { ptr::write_bytes(ptr as *mut u8, 0, usize::try_from(Size4KiB::SIZE).unwrap()) }

        Self {
            queue: unsafe { slice::from_raw_parts_mut(ptr, NUM_OF_TRB_IN_QUEUE) },
        }
    }

    pub fn addr(&self) -> VirtAddr {
        VirtAddr::new(self.queue.as_ptr() as u64)
    }
}

#[repr(transparent)]
struct RawTrb<T: TRB> {
    trb: [u32; 4],
    _marker: PhantomData<T>,
}
