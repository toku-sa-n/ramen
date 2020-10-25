// SPDX-License-Identifier: GPL-3.0-or-later

// WORKAROUND: https://stackoverflow.com/questions/63933070/clippy-says-too-many-arguments-to-static-declaration
#![allow(clippy::too_many_arguments)]

use {
    super::super::paging::pml4::PML4,
    common::constant::{BYTES_KERNEL_HEAP, KERNEL_HEAP_ADDR},
    core::{alloc::Layout, convert::TryFrom},
    linked_list_allocator::LockedHeap,
    uefi::table::boot,
    x86_64::{
        structures::paging::{
            FrameAllocator, Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB,
        },
        PhysAddr,
    },
};

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Using UEFI's `allocate_pages` doesn't work for allocating larger memory. It returns out of
// resrouces.
pub fn init(mem_map: &mut [boot::MemoryDescriptor]) {
    let mut temp_phys_allocator = TemporaryFrameAllocator(mem_map);
    for i in 0..BYTES_KERNEL_HEAP.as_num_of_pages::<Size4KiB>().as_usize() {
        let frame = temp_phys_allocator
            .allocate_frame()
            .expect("OOM during initializing heap area!");

        let page =
            Page::<Size4KiB>::containing_address(KERNEL_HEAP_ADDR + Size4KiB::SIZE * i as u64);
        unsafe {
            PML4.lock()
                .map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT,
                    &mut temp_phys_allocator,
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
