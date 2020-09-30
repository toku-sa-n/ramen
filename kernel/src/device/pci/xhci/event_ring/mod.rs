// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{accessor::slice, mem::allocator::phys::FRAME_MANAGER},
    core::mem::size_of,
    os_units::Size,
    x86_64::structures::paging::{FrameAllocator, PageSize, Size4KiB},
};

#[allow(clippy::cast_possible_truncation)]
const NUM_ELEMENTS_SEGMENT_TABLE: usize = Size4KiB::SIZE as usize / size_of::<SegmentTableEntry>();

struct SegmentTable<'a> {
    table: slice::Accessor<'a, SegmentTableEntry>,
}
impl<'a> SegmentTable<'a> {
    fn new() -> Self {
        let phys_frame = FRAME_MANAGER.lock().allocate_frame().unwrap();
        let phys_addr = phys_frame.start_address();
        let table = slice::Accessor::new(phys_addr, Size::new(0), NUM_ELEMENTS_SEGMENT_TABLE);
        Self { table }
    }
}

struct SegmentTableEntry {
    base_address: u64,
    segment_size: u64,
}
