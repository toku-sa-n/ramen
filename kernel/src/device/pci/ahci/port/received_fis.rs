// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::allocator::page_box::PageBox;

// This is a temporary implementation.
pub struct ReceivedFis(PageBox<u32>);
impl ReceivedFis {
    pub fn new() -> Self {
        Self(PageBox::new(0))
    }
}
