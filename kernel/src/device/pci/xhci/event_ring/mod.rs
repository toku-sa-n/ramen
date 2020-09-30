// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{accessor::slice, mem::allocator::phys::FRAME_MANAGER},
    core::mem::size_of,
    os_units::Size,
    x86_64::structures::paging::{FrameAllocator, PageSize, Size4KiB},
};

#[allow(clippy::cast_possible_truncation)]
const NUM_ELEMENTS_SEGMENT_TABLE: usize = Size4KiB::SIZE as usize / size_of::<SegmentTableEntry>();

pub struct SegmentTable<'a> {
    table: slice::Accessor<'a, SegmentTableEntry>,
}
impl<'a> SegmentTable<'a> {
    pub fn new() -> Self {
        let phys_frame = FRAME_MANAGER.lock().allocate_frame().unwrap();
        let phys_addr = phys_frame.start_address();
        let table = slice::Accessor::new(phys_addr, Size::new(0), NUM_ELEMENTS_SEGMENT_TABLE);
        Self { table }
    }
    pub fn edit<T, U>(&mut self, f: T) -> U
    where
        T: Fn(&mut [SegmentTableEntry]) -> U,
    {
        f(&mut *self.table)
    }
}

pub struct SegmentTableEntry {
    base_address: u64,
    segment_size: u64,
}
impl SegmentTableEntry {
    pub fn set_base_address(&mut self, base_address: u64) {
        self.base_address = base_address
    }

    pub fn set_segment_size(&mut self, segment_size: u16) {
        self.segment_size = u64::from(segment_size)
    }
}
