// SPDX-License-Identifier: GPL-3.0-or-later

// A workaround. Remove this line to check what happens.
#![allow(clippy::too_many_arguments)]

use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use linked_list_allocator::LockedHeap;
use page_box::PageBox;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static HEAP: OnceCell<PageBox<[u8]>> = OnceCell::uninit();

const HEAP_SIZE: usize = 4096 * 12;

pub fn init() {
    allocate_heap();
    init_allocator();
}

fn allocate_heap() {
    HEAP.try_init_once(|| PageBox::new_slice(0, HEAP_SIZE))
        .expect("Failed to initialize `HEAP`.");
}

fn init_allocator() {
    let h = HEAP.try_get().expect("`HEAP` is not initialized.");
    unsafe {
        ALLOCATOR.lock().init(
            h.virt_addr().as_u64().try_into().unwrap(),
            h.bytes().as_usize(),
        )
    }
}
