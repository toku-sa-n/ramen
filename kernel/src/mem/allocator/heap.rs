// SPDX-License-Identifier: GPL-3.0-or-later

// WORKAROUND: https://stackoverflow.com/questions/63933070/clippy-says-too-many-arguments-to-static-declaration
#![allow(clippy::too_many_arguments)]

use super::super::paging::pml4::PML4;
use common::constant::{BYTES_KERNEL_HEAP, KERNEL_HEAP_ADDR};
use core::{alloc::Layout, convert::TryInto};
use linked_list_allocator::LockedHeap;
use uefi::table::boot;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr,
};

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Using UEFI's `allocate_pages` doesn't work for allocating larger memory. It returns out of
// resrouces.
pub fn init(mem_map: &mut [boot::MemoryDescriptor]) {
    alloc_for_heap(mem_map);
    init_allocator();
}

fn alloc_for_heap(mem_map: &mut [boot::MemoryDescriptor]) {
    let mut temp_allocator = TemporaryFrameAllocator(mem_map);
    for i in 0..BYTES_KERNEL_HEAP.as_num_of_pages::<Size4KiB>().as_usize() {
        let frame = temp_allocator
            .allocate_frame()
            .expect("OOM during initializing heap area!");

        let page =
            Page::<Size4KiB>::containing_address(KERNEL_HEAP_ADDR + Size4KiB::SIZE * i as u64);
        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            PML4.lock()
                .map_to(page, frame, flags, &mut temp_allocator)
                .unwrap()
                .flush();
        };
    }
}

fn init_allocator() {
    unsafe {
        ALLOCATOR.lock().init(
            KERNEL_HEAP_ADDR.as_u64().try_into().unwrap(),
            BYTES_KERNEL_HEAP.as_usize(),
        )
    }
}

struct TemporaryFrameAllocator<'a>(&'a mut [boot::MemoryDescriptor]);
unsafe impl<'a> FrameAllocator<Size4KiB> for TemporaryFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for desc in self.0.iter_mut() {
            if desc.ty == boot::MemoryType::CONVENTIONAL && desc.page_count > 0 {
                let frame =
                    PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(desc.phys_start))
                        .unwrap();
                desc.phys_start += Size4KiB::SIZE;
                desc.page_count -= 1;

                return Some(frame);
            }
        }

        None
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocation failed! {:?}", layout);
}
