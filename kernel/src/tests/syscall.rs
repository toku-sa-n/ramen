// SPDX-License-Identifier: GPL-3.0-or-later

use page_box::PageBox;

pub(super) fn main() {
    test_translate_address();
}

fn test_translate_address() {
    let p = PageBox::new(0_i32);
    assert_eq!(p.phys_addr(), syscalls::translate_address(p.virt_addr()));
}
