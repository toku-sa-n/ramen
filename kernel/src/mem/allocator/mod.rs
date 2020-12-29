// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryFrom;
use os_units::NumOfPages;
use phys::FRAME_MANAGER;
use x86_64::{
    structures::paging::{Mapper, Page, PageSize, Size4KiB, Translate},
    PhysAddr, VirtAddr,
};

use super::paging::pml4::PML4;

pub mod acpi;
pub mod heap;
pub mod page_box;
pub mod phys;
pub mod virt;

pub fn allocate_pages(num_of_pages: NumOfPages<Size4KiB>) -> Option<VirtAddr> {
    let phys_addr = allocate_phys(num_of_pages)?;

    let virt_addr = super::map_pages(phys_addr, num_of_pages.as_bytes());

    Some(virt_addr)
}

pub fn deallocate_pages(virt: VirtAddr, num_of_pages: NumOfPages<Size4KiB>) {
    deallocate_phys(virt);
    deallocate_virt(virt, num_of_pages);
}

fn allocate_phys(num_of_pages: NumOfPages<Size4KiB>) -> Option<PhysAddr> {
    FRAME_MANAGER.lock().alloc(num_of_pages)
}

fn deallocate_phys(virt: VirtAddr) {
    let phys = PML4.lock().translate_addr(virt).unwrap();
    FRAME_MANAGER.lock().free(phys);
}

fn deallocate_virt(virt: VirtAddr, num_of_pages: NumOfPages<Size4KiB>) {
    for i in 0..u64::try_from(num_of_pages.as_usize()).unwrap() {
        let page = Page::<Size4KiB>::from_start_address(virt + Size4KiB::SIZE * i).unwrap();

        let (_, flush) = PML4.lock().unmap(page).unwrap();
        flush.flush();
    }
}
