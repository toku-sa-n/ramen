// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::paging::pml4::PML4,
    common::constant::LIMIT_VIRT_ADDR,
    core::convert::TryFrom,
    x86_64::{
        structures::paging::{MapperAllSizes, Page, PageSize, Size4KiB},
        VirtAddr,
    },
};

// TODO: Deallocate after calling passed closure.
pub fn map_temporary<T>(f: T)
where
    T: Fn(VirtAddr),
{
    match search_first_unused_page() {
        Some(addr) => f(addr.start_address()),
        None => panic!("OOM during `map_temporary`"),
    }
}

pub fn search_first_unused_page() -> Option<Page> {
    for addr in (0..LIMIT_VIRT_ADDR.as_u64()).step_by(usize::try_from(Size4KiB::SIZE).unwrap()) {
        let virt_addr = VirtAddr::new(addr);
        if PML4.lock().translate_addr(virt_addr).is_none() {
            return Some(Page::containing_address(virt_addr));
        }
    }
    None
}
