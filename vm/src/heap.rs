#![allow(clippy::too_many_arguments)]

use linked_list_allocator::LockedHeap;

extern "C" {
    static HEAP_START: usize;
    static HEAP_END: usize;
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Using UEFI's `allocate_pages` doesn't work for allocating larger memory. It returns out of
// resrouces.
pub(super) fn init() {
    let s: *const usize = unsafe { &HEAP_START };
    let s = s as usize;

    let e: *const usize = unsafe { &HEAP_END };
    let e = e as usize;

    unsafe { ALLOCATOR.lock().init(s, e - s) }
}
