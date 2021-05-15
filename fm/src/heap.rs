#![allow(clippy::too_many_arguments)]

use linked_list_allocator::LockedHeap;

extern "C" {
    static HEAP_START: usize;
    static HEAP_END: usize;
}

#[global_allocator]
pub(crate) static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub(crate) fn init() {
    let s: *const usize = unsafe { &HEAP_START };
    let s = s as usize;

    let e: *const usize = unsafe { &HEAP_END };
    let e = e as usize;

    unsafe { ALLOCATOR.lock().init(s, e - s) }
}
