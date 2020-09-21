// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::paging::pml4::PML4,
    common::constant::BYTES_AVAILABLE_RAM,
    core::convert::TryFrom,
    os_units::{NumOfPages, Size},
    x86_64::{
        structures::paging::{MapperAllSizes, Page, PageSize, Size4KiB},
        VirtAddr,
    },
};

pub fn search_first_unused_page() -> Option<Page> {
    info!("Searching free virtual address...");
    for addr in
        (0..BYTES_AVAILABLE_RAM.as_usize()).step_by(usize::try_from(Size4KiB::SIZE).unwrap())
    {
        let virt_addr = VirtAddr::new(addr as _);
        if available(virt_addr) {
            info!("Found: {:?}", virt_addr);
            return Some(Page::containing_address(virt_addr));
        }
    }
    None
}

pub fn search_free_addr(num_pages: Size<NumOfPages<Size4KiB>>) -> Option<VirtAddr> {
    let mut cnt = 0;
    let mut start = None;
    for addr in
        (0..BYTES_AVAILABLE_RAM.as_usize()).step_by(usize::try_from(Size4KiB::SIZE).unwrap())
    {
        let addr = VirtAddr::new(addr as _);
        if available(addr) {
            if start.is_none() {
                start = Some(addr);
            }

            cnt += 1;

            if cnt >= num_pages.as_usize() {
                return start;
            }
        } else {
            cnt = 0;
            start = None;
        }
    }

    None
}

fn available(addr: VirtAddr) -> bool {
    PML4.lock().translate_addr(addr).is_none() && addr != VirtAddr::zero()
}
