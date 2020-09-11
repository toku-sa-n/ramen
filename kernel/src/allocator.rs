// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::too_many_arguments)]
use common::constant::{CHANGE_FREE_PAGE_ADDR, FREE_PAGE_ADDR};
use conquer_once::spin::Lazy;
use core::{alloc::Layout, ptr};
use linked_list_allocator::LockedHeap;
use spinning_top::Spinlock;
use uefi::table::{boot, boot::MemoryType};
use x86_64::{
    instructions::tlb,
    structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB},
    PhysAddr,
};

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static FRAME_MANAGER: Lazy<Spinlock<FrameManager>> = Lazy::new(|| {
    Spinlock::new(FrameManager {
        head: None,
        tail: None,
    })
});

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
