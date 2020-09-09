// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::too_many_arguments)]
use core::alloc::Layout;
use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocation failed! {:?}", layout);
}
