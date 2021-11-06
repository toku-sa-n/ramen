// SPDX-License-Identifier: GPL-3.0-or-later

// WORKAROUND: https://stackoverflow.com/questions/63933070/clippy-says-too-many-arguments-to-static-declaration
#![allow(clippy::too_many_arguments)]

use {core::alloc::Layout, linked_list_allocator::LockedHeap};

extern "C" {
    static HEAP_START: usize;
    static HEAP_END: usize;
}

#[global_allocator]
pub(crate) static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Using UEFI's `allocate_pages` doesn't work for allocating larger memory. It returns out of
// resrouces.
pub(crate) fn init() {
    let s: *const usize = unsafe { &HEAP_START };
    let s = s as usize;

    let e: *const usize = unsafe { &HEAP_END };
    let e = e as usize;

    unsafe { ALLOCATOR.lock().init(s, e - s) }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocation failed! {:?}", layout);
}
