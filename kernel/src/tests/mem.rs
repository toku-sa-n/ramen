// SPDX-License-Identifier: GPL-3.0-or-later

use page_box::PageBox;

pub(super) fn main() {
    test_page_box_clone();
}

fn test_page_box_clone() {
    let b = PageBox::new(3);
    let b2 = b.clone();

    assert_eq!(*b, *b2);

    let b = PageBox::new_slice(334, 5);
    let b2 = b.clone();

    assert_eq!(*b, *b2);
}
