// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::allocator::page_box::PageBox,
    core::ops::{Index, IndexMut},
    x86_64::PhysAddr,
};

pub struct SegmentTable(PageBox<[Entry]>);
impl SegmentTable {
    fn phys_addr(&self) -> PhysAddr {
        self.0.phys_addr()
    }
}
impl Index<usize> for SegmentTable {
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for SegmentTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[repr(C, packed)]
struct Entry {
    base_address: u64,
    segment_size: u64,
}
impl Entry {
    // Although the size of segment_size is u64, bits 16:63 are reserved.
    pub fn set(&mut self, addr: PhysAddr, size: u16) {
        self.base_address = addr.as_u64();
        self.segment_size = u64::from(size);
    }
}
