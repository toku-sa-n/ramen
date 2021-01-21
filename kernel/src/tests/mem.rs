// SPDX-License-Identifier: GPL-3.0-or-later

use page_box::PageBox;

pub(super) fn main() {
    test_page_box_clone();
    test_page_box_from_slice();
}

fn test_page_box_clone() {
    let b = PageBox::from(3);
    let b2 = b.clone();

    assert_eq!(*b, *b2);

    let b = PageBox::new_slice(334, 5);
    let b2 = b.clone();

    assert_eq!(*b, *b2);
}

fn test_page_box_from_slice() {
    let s: &[i32] = &[3, 3, 4];
    let b = PageBox::from(s);

    assert_eq!(*b, *s);
}
