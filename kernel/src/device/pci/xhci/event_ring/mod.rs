// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::allocator::page_box::PageBox,
    core::mem::size_of,
    x86_64::{
        structures::paging::{PageSize, Size4KiB},
        PhysAddr,
    },
};

#[allow(clippy::cast_possible_truncation)]
pub const NUM_ELEMENTS_SEGMENT_TABLE: usize =
    Size4KiB::SIZE as usize / size_of::<SegmentTableEntry>();

pub struct SegmentTable {
    table: PageBox<[SegmentTableEntry]>,
}
impl SegmentTable {
    pub fn new() -> Self {
        let table = PageBox::new_slice(NUM_ELEMENTS_SEGMENT_TABLE);
        Self { table }
    }

    pub fn edit<T, U>(&mut self, f: T) -> U
    where
        T: Fn(&mut [SegmentTableEntry]) -> U,
    {
        f(&mut *self.table)
    }

    pub fn addr(&self) -> PhysAddr {
        self.table.phys_addr()
    }
}

#[repr(C, packed)]
pub struct SegmentTableEntry {
    base_address: u64,
    segment_size: u64,
}
impl SegmentTableEntry {
    pub fn set_base_address(&mut self, base_address: PhysAddr) {
        self.base_address = base_address.as_u64()
    }

    pub fn set_segment_size(&mut self, segment_size: u16) {
        self.segment_size = u64::from(segment_size)
    }
}
