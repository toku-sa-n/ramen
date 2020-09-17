// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{super::paging::pml4::PML4, phys::FRAME_MANAGER},
    common::constant::{BYTES_KERNEL_HEAP, KERNEL_HEAP_ADDR},
    core::{alloc::Layout, convert::TryFrom},
    linked_list_allocator::LockedHeap,
    x86_64::structures::paging::{
        FrameAllocator, Mapper, Page, PageSize, PageTableFlags, Size4KiB,
    },
};

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Using UEFI's `allocate_pages` doesn't work for allocating larger memory. It returns out of
// resrouces.
pub fn init() {
    for i in 0..BYTES_KERNEL_HEAP.as_num_of_pages::<Size4KiB>().as_usize() {
        let frame = FRAME_MANAGER
            .lock()
            .allocate_frame()
            .expect("OOM during initializing heap memory.");
        let page =
            Page::<Size4KiB>::containing_address(KERNEL_HEAP_ADDR + Size4KiB::SIZE * i as u64);
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
        };
    }

    unsafe {
        ALLOCATOR.lock().init(
            usize::try_from(KERNEL_HEAP_ADDR.as_u64()).unwrap(),
            BYTES_KERNEL_HEAP.as_usize(),
        )
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocation failed! {:?}", layout);
}
