// SPDX-License-Identifier: GPL-3.0-or-later

use os_units::NumOfPages;
use x86_64::{structures::paging::Size4KiB, VirtAddr};

#[cfg(not(test))]
pub(super) fn allocate_pages(n: NumOfPages<Size4KiB>) -> VirtAddr {
    let v = syscalls::allocate_pages(n);

    if v.is_null() {
        panic!("Failed to allocate pages.");
    }

    v
}

#[cfg(test)]
pub(super) fn allocate_pages(n: NumOfPages<Size4KiB>) -> VirtAddr {
    use std::{alloc, alloc::Layout, convert::TryInto};
    use x86_64::structures::paging::PageSize;

    let sz: usize = Size4KiB::SIZE.try_into().unwrap();
    let l = Layout::from_size_align(sz, sz);
    let l = l.expect("Invalid layout.");

    let p = unsafe { alloc::alloc(l) };
    VirtAddr::from_ptr(p)
}

#[cfg(not(test))]
pub(super) fn deallocate_pages(v: VirtAddr, n: NumOfPages<Size4KiB>) {
    syscalls::deallocate_pages(v, n);
}

#[cfg(test)]
pub(super) fn deallocate_pages(v: VirtAddr, n: NumOfPages<Size4KiB>) {
    use std::{alloc, alloc::Layout, convert::TryInto};
    use x86_64::structures::paging::PageSize;

    let sz: usize = Size4KiB::SIZE.try_into().unwrap();
    let l = Layout::from_size_align(sz, sz);
    let l = l.expect("Invalid layout.");

    let p = v.as_mut_ptr();
    unsafe { alloc::dealloc(p, l) }
}
