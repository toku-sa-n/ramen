// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(test), no_std)]
#![feature(int_bits_const)]

extern crate alloc;

use alloc::vec::Vec;
use boot::MemoryType;
use core::{convert::TryInto, fmt};
use os_units::NumOfPages;
use uefi::table::boot;
use x86_64::{
    structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB},
    PhysAddr,
};

#[derive(PartialEq, Eq, Debug)]
pub struct FrameManager(Vec<Frames>);
impl FrameManager {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn alloc(&mut self, num_of_pages: NumOfPages<Size4KiB>) -> Option<PhysAddr> {
        for i in 0..self.0.len() {
            if self.0[i].is_available_for_allocating(num_of_pages) {
                return Some(self.alloc_from_descriptor_index(i, num_of_pages));
            }
        }

        None
    }

    pub fn free(&mut self, addr: PhysAddr) {
        for i in 0..self.0.len() {
            if self.0[i].start == addr && !self.0[i].available {
                return self.free_memory_for_descriptor_index(i);
            }
        }
    }

    pub fn init(&mut self, mem_map: &[boot::MemoryDescriptor]) {
        for descriptor in mem_map {
            if Self::available(descriptor.ty) {
                self.init_for_descriptor(descriptor);
            }
        }
    }

    fn alloc_from_descriptor_index(&mut self, i: usize, n: NumOfPages<Size4KiB>) -> PhysAddr {
        self.split_node(i, n);

        self.0[i].available = false;
        self.0[i].start
    }

    fn init_for_descriptor(&mut self, descriptor: &boot::MemoryDescriptor) {
        let start = PhysAddr::new(descriptor.phys_start);
        let num = NumOfPages::new(descriptor.page_count.try_into().unwrap());
        let frames = Frames::new_for_available(start, num);

        self.0.push(frames);
    }

    fn split_node(&mut self, i: usize, num_of_pages: NumOfPages<Size4KiB>) {
        assert!(self.0[i].available, "Frames are not available.");
        assert!(
            self.0[i].num_of_pages > num_of_pages,
            "Insufficient number of frames."
        );

        self.split_node_unchecked(i, num_of_pages)
    }

    fn split_node_unchecked(&mut self, i: usize, requested: NumOfPages<Size4KiB>) {
        let new_frames_start = self.0[i].start + requested.as_bytes().as_usize();
        let new_frames_num = self.0[i].num_of_pages - requested;
        let new_frames = Frames::new_for_available(new_frames_start, new_frames_num);

        self.0[i].num_of_pages = requested;
        self.0.insert(i + 1, new_frames);
    }

    fn free_memory_for_descriptor_index(&mut self, i: usize) {
        self.0[i].available = true;
        self.merge_before_and_after_frames(i);
    }

    fn merge_before_and_after_frames(&mut self, i: usize) {
        if self.mergeable_to_next_frames(i) {
            self.merge_to_next_frames(i);
        }

        if self.mergeable_to_next_frames(i - 1) {
            self.merge_to_next_frames(i - 1);
        }
    }

    fn mergeable_to_next_frames(&self, i: usize) -> bool {
        if i >= self.0.len() - 1 {
            return false;
        }

        let node = &self.0[i];
        let next = &self.0[i + 1];

        node.is_mergeable(next)
    }

    fn merge_to_next_frames(&mut self, i: usize) {
        self.merge_two_nodes(i);
    }

    fn merge_two_nodes(&mut self, i: usize) {
        let n = self.0[i + 1].num_of_pages;
        self.0[i].num_of_pages += n;
        self.0.remove(i + 1);
    }

    fn available(ty: boot::MemoryType) -> bool {
        ty == MemoryType::CONVENTIONAL
    }
}
unsafe impl FrameAllocator<Size4KiB> for FrameManager {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let addr = self.alloc(NumOfPages::new(1))?;
        Some(PhysFrame::from_start_address(addr).unwrap())
    }
}
impl FrameDeallocator<Size4KiB> for FrameManager {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let addr = frame.start_address();
        self.free(addr);
    }
}

#[derive(PartialEq, Eq)]
struct Frames {
    start: PhysAddr,
    num_of_pages: NumOfPages<Size4KiB>,
    available: bool,
}
impl Frames {
    fn new_for_available(start: PhysAddr, num_of_pages: NumOfPages<Size4KiB>) -> Self {
        Self {
            start,
            num_of_pages,
            available: true,
        }
    }

    #[cfg(test)]
    fn new_for_used(start: PhysAddr, num_of_pages: NumOfPages<Size4KiB>) -> Self {
        Self {
            start,
            num_of_pages,
            available: false,
        }
    }

    fn is_available_for_allocating(&self, request_num_of_pages: NumOfPages<Size4KiB>) -> bool {
        self.num_of_pages >= request_num_of_pages && self.available
    }

    fn is_mergeable(&self, other: &Self) -> bool {
        self.available && other.available && self.is_consecutive(other) && self.is_same_size(other)
    }

    fn is_consecutive(&self, other: &Self) -> bool {
        self.end() == other.start
    }

    fn end(&self) -> PhysAddr {
        self.start + self.num_of_pages.as_bytes().as_usize()
    }

    fn is_same_size(&self, other: &Self) -> bool {
        self.num_of_pages == other.num_of_pages
    }
}
impl fmt::Debug for Frames {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.available { "Available" } else { "Used" };
        write!(
            f,
            "Frames::<{}>({:?} .. {:?})",
            suffix,
            self.start,
            self.end()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameManager, Frames};
    use os_units::NumOfPages;
    use x86_64::PhysAddr;

    #[test]
    fn fail_to_allocate() {
        let mut f = frame_manager_for_testing();

        let a = f.alloc(NumOfPages::new(200));
        assert!(a.is_none());
    }

    #[test]
    fn allocate_not_power_of_two() {
        let mut f = frame_manager_for_testing();

        let a = f.alloc(NumOfPages::new(3));

        assert_eq!(a, Some(PhysAddr::new(0x2000)));
        assert_eq!(
            f,
            FrameManager(vec![
                Frames::new_for_available(PhysAddr::zero(), NumOfPages::new(1)),
                Frames::new_for_used(PhysAddr::new(0x2000), NumOfPages::new(3)),
                Frames::new_for_available(PhysAddr::new(0x5000), NumOfPages::new(7)),
                Frames::new_for_used(PhysAddr::new(0xc000), NumOfPages::new(4)),
            ])
        );
    }

    #[test]
    fn free() {
        let mut f = frame_manager_for_testing();

        f.free(PhysAddr::new(0xc000));

        assert_eq!(
            f,
            FrameManager(vec![
                Frames::new_for_available(PhysAddr::zero(), NumOfPages::new(1)),
                Frames::new_for_available(PhysAddr::new(0x2000), NumOfPages::new(14))
            ])
        );
    }

    fn frame_manager_for_testing() -> FrameManager {
        let f = vec![
            Frames::new_for_available(PhysAddr::zero(), NumOfPages::new(1)),
            Frames::new_for_available(PhysAddr::new(0x2000), NumOfPages::new(10)),
            Frames::new_for_used(PhysAddr::new(0xc000), NumOfPages::new(4)),
        ];

        FrameManager(f)
    }
}
