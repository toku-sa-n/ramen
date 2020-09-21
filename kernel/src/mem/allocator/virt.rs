// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::paging::pml4::PML4,
    common::constant::BYTES_AVAILABLE_RAM,
    core::convert::TryFrom,
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

fn available(addr: VirtAddr) -> bool {
    PML4.lock().translate_addr(addr).is_none() && addr != VirtAddr::zero()
}
