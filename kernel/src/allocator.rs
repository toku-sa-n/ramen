// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::too_many_arguments)]
use {
    crate::mem::paging::pml4::PML4,
    common::constant::{
        BYTES_KERNEL_HEAP, CHANGE_FREE_PAGE_ADDR, FREE_PAGE_ADDR, KERNEL_HEAP_ADDR,
    },
    conquer_once::spin::{Lazy, OnceCell},
    core::{alloc::Layout, convert::TryFrom, ptr},
    linked_list_allocator::LockedHeap,
    spinning_top::Spinlock,
    uefi::table::{boot, boot::MemoryType},
    x86_64::{
        instructions::tlb,
        structures::paging::{
            FrameAllocator, Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB,
        },
        PhysAddr,
    },
};

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static FRAME_MANAGER: Lazy<Spinlock<FrameManager>> = Lazy::new(|| {
    Spinlock::new(FrameManager {
        head: None,
        tail: None,
    })
});

// Using UEFI's `allocate_pages` doesn't work for allocating larger memory. It returns out of
// resrouces.
pub fn init_heap() {
    for i in 0..BYTES_KERNEL_HEAP.as_num_of_pages::<Size4KiB>().as_usize() {
        let frame = FRAME_MANAGER
            .lock()
            .allocate_frame()
            .expect("OOM during initializing heap memory.");
        let page =
            Page::<Size4KiB>::containing_address(KERNEL_HEAP_ADDR + Size4KiB::SIZE * i as u64);
        unsafe {
            PML4.lock()
                .map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT,
                    &mut *FRAME_MANAGER.lock(),
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

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocation failed! {:?}", layout);
}

pub struct FrameManager {
    head: Option<PhysAddr>,
    tail: Option<PhysAddr>,
}

impl FrameManager {
    pub fn init(mem_map: &[boot::MemoryDescriptor]) {
        FRAME_MANAGER.lock().init_static(mem_map);
    }

    fn init_static(&mut self, mem_map: &[boot::MemoryDescriptor]) {
        for descriptor in mem_map {
            if Self::available(descriptor.ty) {
                self.init_for_descriptor(descriptor);
            }
        }
    }

    fn init_for_descriptor(&mut self, descriptor: &boot::MemoryDescriptor) {
        for i in 0..descriptor.page_count {
            let addr = PhysAddr::new(descriptor.phys_start + Size4KiB::SIZE * i);
            if addr.is_null() {
                continue;
            }

            if self.head.is_none() {
                self.head = Some(addr);
            }

            unsafe {
                ptr::write(addr.as_u64() as *mut Option<PhysAddr>, None);
            }

            if let Some(prev) = self.tail {
                unsafe { ptr::write(prev.as_u64() as _, Some(addr)) }
            }

            self.tail = Some(addr);
        }
    }

    fn change_free_page_ptr(addr: PhysAddr) {
        const PAGE_EXISTS: u64 = 1;
        unsafe {
            ptr::write(
                CHANGE_FREE_PAGE_ADDR.as_mut_ptr(),
                addr.as_u64() | PAGE_EXISTS,
            )
        }
        tlb::flush(CHANGE_FREE_PAGE_ADDR);
    }

    fn available(ty: boot::MemoryType) -> bool {
        ty == MemoryType::CONVENTIONAL
    }
}

unsafe impl FrameAllocator<Size4KiB> for FrameManager {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        match self.head {
            None => None,
            Some(addr) => {
                Self::change_free_page_ptr(addr);
                unsafe {
                    self.head = ptr::read(addr.as_u64() as _);
                }

                Some(PhysFrame::containing_address(addr))
            }
        }
    }
}
