// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryFrom;
use os_units::NumOfPages;
use phys::FRAME_MANAGER;
use x86_64::{
    structures::paging::{
        FrameDeallocator, Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB,
    },
    VirtAddr,
};

use super::paging::pml4::PML4;

pub mod acpi;
pub mod heap;
pub mod page_box;
pub mod phys;
pub mod virt;

pub fn allocate_pages(num_of_pages: NumOfPages<Size4KiB>) -> VirtAddr {
    let virt_addr = virt::search_free_addr(num_of_pages).expect("OOM during creating `PageBox`");

    let phys_addr = FRAME_MANAGER
        .lock()
        .alloc(num_of_pages)
        .expect("OOM during creating `PageBox");

    for i in 0..u64::try_from(num_of_pages.as_usize()).unwrap() {
        let page = Page::<Size4KiB>::from_start_address(virt_addr + Size4KiB::SIZE * i).unwrap();
        let frame = PhysFrame::from_start_address(phys_addr + Size4KiB::SIZE * i).unwrap();
        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        let f = &mut *FRAME_MANAGER.lock();

        unsafe { PML4.lock().map_to(page, frame, flags, f).unwrap().flush() }
    }

    virt_addr
}

pub fn deallocate_pages(virt: VirtAddr, num_of_pages: NumOfPages<Size4KiB>) {
    for i in 0..u64::try_from(num_of_pages.as_usize()).unwrap() {
        let page = Page::from_start_address(virt + Size4KiB::SIZE * i).unwrap();

        let (frame, flush) = PML4.lock().unmap(page).unwrap();
        flush.flush();
        unsafe { FRAME_MANAGER.lock().deallocate_frame(frame) }
    }
}
