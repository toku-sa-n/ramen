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
    for addr in
        (0..BYTES_AVAILABLE_RAM.as_usize()).step_by(usize::try_from(Size4KiB::SIZE).unwrap())
    {
        let virt_addr = VirtAddr::new(addr as _);
        if PML4.lock().translate_addr(virt_addr).is_none() {
            return Some(Page::containing_address(virt_addr));
        }
    }
    None
}
