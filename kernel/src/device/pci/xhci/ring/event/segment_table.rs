// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::accessor::slice,
    core::ops::{Index, IndexMut},
    x86_64::PhysAddr,
};

pub struct SegmentTable<'a> {
    addr: PhysAddr,
    table: slice::Accessor<'a, Entry>,
}
impl<'a> Index<usize> for SegmentTable<'a> {
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}
impl<'a> IndexMut<usize> for SegmentTable<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.table[index]
    }
}

#[repr(C, packed)]
struct Entry {
    base_address: u64,
    segment_size: u64,
}
impl Entry {
    // Although the size of segment_size is u64, bits 16:63 are reserved.
    fn set(&mut self, addr: PhysAddr, size: u16) {
        self.base_address = addr.as_u64();
        self.segment_size = u64::from(size);
    }
}
