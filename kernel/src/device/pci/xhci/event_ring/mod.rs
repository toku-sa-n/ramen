// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::{accessor::slice, allocator::phys::FRAME_MANAGER},
    core::mem::size_of,
    os_units::Bytes,
    x86_64::{
        structures::paging::{FrameAllocator, PageSize, Size4KiB},
        PhysAddr,
    },
};

#[allow(clippy::cast_possible_truncation)]
pub const NUM_ELEMENTS_SEGMENT_TABLE: usize =
    Size4KiB::SIZE as usize / size_of::<SegmentTableEntry>();

pub struct SegmentTable<'a> {
    addr: PhysAddr,
    table: slice::Accessor<'a, SegmentTableEntry>,
}
impl<'a> SegmentTable<'a> {
    pub fn new() -> Self {
        let phys_frame = FRAME_MANAGER.lock().allocate_frame().unwrap();
        let addr = phys_frame.start_address();
        let table = slice::Accessor::new(addr, Bytes::new(0), NUM_ELEMENTS_SEGMENT_TABLE);
        Self { addr, table }
    }

    pub fn edit<T, U>(&mut self, f: T) -> U
    where
        T: Fn(&mut [SegmentTableEntry]) -> U,
    {
        f(&mut *self.table)
    }

    pub fn addr(&self) -> PhysAddr {
        self.addr
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
