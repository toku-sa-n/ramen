// SPDX-License-Identifier: GPL-3.0-or-later

// WORKAROUND: https://stackoverflow.com/questions/63933070/clippy-says-too-many-arguments-to-static-declaration
#![allow(clippy::too_many_arguments)]

use core::alloc::Layout;
use linked_list_allocator::LockedHeap;

extern "C" {
    static HEAP_START: usize;
    static HEAP_END: usize;
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Using UEFI's `allocate_pages` doesn't work for allocating larger memory. It returns out of
// resrouces.
pub fn init() {
    let s = unsafe { &HEAP_START as *const usize as usize };
    let e = unsafe { &HEAP_END as *const usize as usize };

    unsafe { ALLOCATOR.lock().init(s, e - s) }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocation failed! {:?}", layout);
}
