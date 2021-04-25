// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![allow(clippy::too_many_arguments)] // A workaround for the clippy's wrong warning.
#![feature(alloc_error_handler)]

pub mod io;
pub mod mem;

extern crate alloc;

use conquer_once::spin::Lazy;
use core::{alloc::Layout, convert::TryInto};
use linked_list_allocator::LockedHeap;
use os_units::{Bytes, NumOfPages};
use page_box::PageBox;
use x86_64::{structures::paging::Size4KiB, VirtAddr};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static HEAP: Lazy<Heap> = Lazy::new(Heap::default);

struct Heap(PageBox<[u8]>);
impl Heap {
    const NUM_PAGES: NumOfPages<Size4KiB> = NumOfPages::new(16);

    fn start(&self) -> VirtAddr {
        self.0.virt_addr()
    }

    fn bytes(&self) -> Bytes {
        self.0.bytes()
    }
}
impl Default for Heap {
    fn default() -> Self {
        Self(PageBox::new_slice(0, Self::NUM_PAGES.as_bytes().as_usize()))
    }
}

pub fn init() {
    init_heap();
    io::init();
}

fn init_heap() {
    let a = ALLOCATOR.try_lock();
    let mut a = a.expect("Failed to acquire the lock of `HEAP`.");

    let start: usize = HEAP.start().as_u64().try_into().unwrap();
    let bytes = HEAP.bytes().as_usize();

    unsafe { a.init(start, bytes) }
}

#[panic_handler]
fn panic(i: &core::panic::PanicInfo<'_>) -> ! {
    syscalls::panic(i);
}

#[alloc_error_handler]
fn alloc_fail(l: Layout) -> ! {
    panic!("Allocation failed: {:?}", l)
}
