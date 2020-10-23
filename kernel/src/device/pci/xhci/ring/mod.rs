// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::{accessor::slice, allocator::phys::FRAME_MANAGER},
    core::ops::{Index, IndexMut},
    os_units::Size,
    x86_64::structures::paging::{FrameAllocator, PageSize, Size4KiB},
};

mod command;
mod event;
mod trb;

struct Raw<'a>(slice::Accessor<'a, RawTrb>);
impl<'a> Raw<'a> {
    fn new(num_trb: usize) -> Self {
        assert!(num_trb as u64 <= Size4KiB::SIZE / 16);
        let phys_frame = FRAME_MANAGER.lock().allocate_frame().unwrap();
        let addr = phys_frame.start_address();
        Self(slice::Accessor::new(addr, Size::new(0), num_trb))
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<'a> Index<usize> for Raw<'a> {
    type Output = RawTrb;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl<'a> IndexMut<usize> for Raw<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[repr(transparent)]
struct RawTrb(u128);

struct CycleBit(bool);
impl CycleBit {
    fn new(val: bool) -> Self {
        Self(val)
    }

    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}
impl From<CycleBit> for bool {
    fn from(cycle_bit: CycleBit) -> Self {
        cycle_bit.0
    }
}
