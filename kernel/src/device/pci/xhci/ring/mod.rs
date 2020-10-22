// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{accessor::slice, mem::allocator::phys::FRAME_MANAGER},
    core::ops::{Index, IndexMut},
    os_units::Size,
    x86_64::structures::paging::{FrameAllocator, PageSize, Size4KiB},
};

mod event;
mod trb;

struct Raw<'a>(slice::Accessor<'a, u128>);
impl<'a> Raw<'a> {
    fn new(num_trb: usize) -> Self {
        assert!(num_trb as u64 <= Size4KiB::SIZE / 16);
        let phys_frame = FRAME_MANAGER.lock().allocate_frame().unwrap();
        let addr = phys_frame.start_address();
        Self(slice::Accessor::new(addr, Size::new(0), num_trb))
    }
}
impl<'a> Index<usize> for Raw<'a> {
    type Output = u128;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl<'a> IndexMut<usize> for Raw<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
