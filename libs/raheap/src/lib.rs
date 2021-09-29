#![no_std]
// This is a workaround for
// https://stackoverflow.com/questions/63933070/clippy-says-too-many-arguments-to-static-declaration.
#![allow(clippy::too_many_arguments)]

use conquer_once::spin::Lazy;
use core::convert::TryInto;
use linked_list_allocator::LockedHeap;
use os_units::{Bytes, NumOfPages};
use page_box::PageBox;
use x86_64::{structures::paging::Size4KiB, VirtAddr};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static HEAP: Lazy<Heap> = Lazy::new(Heap::default);

struct Heap(PageBox<[u8]>);
impl Heap {
    fn start(&self) -> VirtAddr {
        self.0.virt_addr()
    }

    fn bytes(&self) -> Bytes {
        self.0.bytes()
    }
}
impl Default for Heap {
    fn default() -> Self {
        let num_pages = NumOfPages::<Size4KiB>::new(16);
        Self(PageBox::new_slice(0, num_pages.as_bytes().as_usize()))
    }
}

#[allow(clippy::missing_panics_doc)]
pub fn init() {
    let a = ALLOCATOR.try_lock();
    let mut a = a.expect("Failed to acquire the lock of `HEAP`.");

    let start: usize = HEAP.start().as_u64().try_into().unwrap();
    let bytes = HEAP.bytes().as_usize();

    unsafe { a.init(start, bytes) }
}
