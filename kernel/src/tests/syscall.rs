// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use page_box::PageBox;

pub(super) fn main() {
    test_translate_address();
    test_write();
}

fn test_translate_address() {
    let p = PageBox::from(0_i32);
    assert_eq!(p.phys_addr(), syscalls::translate_address(p.virt_addr()));
}

fn test_write() {
    let s = "Test for the write system call";
    let nbyte = s.len();
    let buf = s.as_ptr();

    // SAFETY: `buf` is valid.
    unsafe { syscalls::write(1, buf.cast(), nbyte.try_into().unwrap()) };
}
