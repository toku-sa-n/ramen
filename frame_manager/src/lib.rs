// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(test), no_std)]
#![feature(int_bits_const)]

use alloc::vec::Vec;
use bit_field::BitField;
use boot::MemoryType;
use core::convert::{TryFrom, TryInto};
use os_units::NumOfPages;
use uefi::table::boot;
use x86_64::{
    structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB},
    PhysAddr,
};

extern crate alloc;

pub struct FrameManager(Vec<Frames>);
impl FrameManager {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn alloc(&mut self, num_of_pages: NumOfPages<Size4KiB>) -> Option<PhysAddr> {
        let num_of_pages = NumOfPages::new(num_of_pages.as_usize().next_power_of_two());

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

        self.merge_all_nodes();
    }

    fn alloc_from_descriptor_index(&mut self, i: usize, n: NumOfPages<Size4KiB>) -> PhysAddr {
        self.split_node(i, n);

        self.0[i].available = false;
        self.0[i].start
    }

    fn init_for_descriptor(&mut self, descriptor: &boot::MemoryDescriptor) {
        let mut offset = NumOfPages::<Size4KiB>::new(0);

        // By reversing the range, bigger memory chanks come first.
        // This will make it faster to search a small amount of memory.
        for i in (0..u64::BITS).rev() {
            if descriptor.page_count.get_bit(i.try_into().unwrap()) {
                let addr = PhysAddr::new(
                    descriptor.phys_start + u64::try_from(offset.as_bytes().as_usize()).unwrap(),
                );
                let pages = NumOfPages::new(2_usize.pow(i));
                let frames = Frames::new_for_available(addr, pages);

                self.0.push(frames);

                offset += pages;
            }
        }
    }

    fn split_node(&mut self, i: usize, num_of_pages: NumOfPages<Size4KiB>) {
        assert!(self.0[i].available, "Frames are not available.");

        while self.0[i].num_of_pages > num_of_pages {
            self.split_node_into_half(i);
        }
    }

    fn split_node_into_half(&mut self, i: usize) {
        let start = self.0[i].start;
        let num_of_pages = self.0[i].num_of_pages;

        let new_frames = Frames::new_for_available(
            start + num_of_pages.as_bytes().as_usize() / 2,
            num_of_pages / 2,
        );

        self.0[i].num_of_pages /= 2;
        self.0.insert(i + 1, new_frames);
    }

    fn free_memory_for_descriptor_index(&mut self, i: usize) {
        self.0[i].available = true;
        self.merge_all_nodes();
    }

    fn merge_all_nodes(&mut self) {
        // By reversing the range, bigger memory chanks come first.
        // This will make it faster to search a small amount of memory.
        for i in (0..self.0.len()).rev() {
            if self.mergeable(i) {
                return self.merge_node(i);
            }
        }
    }

    fn mergeable(&self, i: usize) -> bool {
        if i >= self.0.len() - 1 {
            return false;
        }

        let node = &self.0[i];
        let next = &self.0[i + 1];

        node.is_mergeable(next)
    }

    fn merge_node(&mut self, i: usize) {
        self.merge_two_nodes(i);
        self.merge_all_nodes();
    }

    fn merge_two_nodes(&mut self, i: usize) {
        self.0[i].num_of_pages *= 2;
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

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::{FrameManager, Frames};
    use os_units::NumOfPages;
    use x86_64::PhysAddr;

    #[test]
    fn fail_to_allocate() {
        let mut f = frame_manager_for_testing();

        let a = f.alloc(NumOfPages::new(5));
        assert!(a.is_none());
    }

    fn frame_manager_for_testing() -> FrameManager {
        let f = vec![Frames::new_for_available(
            PhysAddr::zero(),
            NumOfPages::new(1),
        )];

        FrameManager(f)
    }
}
