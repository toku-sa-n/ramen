// SPDX-License-Identifier: GPL-3.0-or-later

use page_box::PageBox;
use x86_64::PhysAddr;

// This is a temporary implementation.
pub struct ReceivedFis(PageBox<u32>);
impl ReceivedFis {
    pub fn new() -> Self {
        Self(0.into())
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.0.phys_addr()
    }
}
