// SPDX-License-Identifier: GPL-3.0-or-later

use os_units::NumOfPages;
use x86_64::{structures::paging::Size4KiB, VirtAddr};

#[cfg(not(test))]
pub(super) fn allocate_pages(n: NumOfPages<Size4KiB>) -> VirtAddr {
    let v = syscalls::allocate_pages(n);

    assert!(!v.is_null(), "Failed to allocate pages.");

    v
}

#[cfg(test)]
pub(super) fn allocate_pages(n: NumOfPages<Size4KiB>) -> VirtAddr {
    let l = num_of_pages_to_layout(n);
    let p = unsafe { std::alloc::alloc(l) };

    VirtAddr::from_ptr(p)
}

#[cfg(not(test))]
pub(super) fn deallocate_pages(v: VirtAddr, n: NumOfPages<Size4KiB>) {
    syscalls::deallocate_pages(v, n);
}

#[cfg(test)]
pub(super) fn deallocate_pages(v: VirtAddr, n: NumOfPages<Size4KiB>) {
    let l = num_of_pages_to_layout(n);
    let p = v.as_mut_ptr();

    unsafe { std::alloc::dealloc(p, l) }
}

#[cfg(test)]
fn num_of_pages_to_layout(n: NumOfPages<Size4KiB>) -> std::alloc::Layout {
    let sz = n.as_bytes().as_usize();
    let l = std::alloc::Layout::from_size_align(sz, sz);

    l.expect("Invalid layout.")
}
