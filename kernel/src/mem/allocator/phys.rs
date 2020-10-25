// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::paging,
    alloc::collections::vec_deque::VecDeque,
    conquer_once::spin::Lazy,
    core::convert::TryFrom,
    os_units::NumOfPages,
    spinning_top::Spinlock,
    uefi::table::boot::{self, MemoryType},
    x86_64::{
        structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB},
        PhysAddr,
    },
};

pub static FRAME_MANAGER: Lazy<Spinlock<FrameManager>> =
    Lazy::new(|| Spinlock::new(FrameManager(VecDeque::new())));

pub struct FrameManager(VecDeque<Frames>);
impl FrameManager {
    pub fn init(mem_map: &[boot::MemoryDescriptor]) {
        FRAME_MANAGER.lock().init_static(mem_map);
        paging::mark_pages_as_unused();
    }

    pub fn alloc(&mut self, num_of_pages: NumOfPages<Size4KiB>) -> Option<PhysAddr> {
        let num_of_pages = NumOfPages::new(num_of_pages.as_usize().next_power_of_two());

        for i in 0..self.0.len() {
            if self.0[i].num_of_pages >= num_of_pages && self.0[i].available {
                self.split_node(i, num_of_pages);

                let addr = self.0[i].start;
                self.0[i].available = false;

                return Some(addr);
            }
        }

        None
    }

    pub fn free(&mut self, addr: PhysAddr) {
        for i in 0..self.0.len() {
            if self.0[i].start == addr && !self.0[i].available {
                self.0[i].available = true;
                return self.merge_all_nodes();
            }
        }
    }

    fn init_static(&mut self, mem_map: &[boot::MemoryDescriptor]) {
        for descriptor in mem_map {
            if Self::available(descriptor.ty) {
                self.init_for_descriptor(descriptor);
            }
        }

        self.merge_all_nodes();
    }

    fn init_for_descriptor(&mut self, descriptor: &boot::MemoryDescriptor) {
        for i in 0..descriptor.page_count {
            let addr = PhysAddr::new(descriptor.phys_start + Size4KiB::SIZE * i);
            let frames = Frames::new(addr, NumOfPages::new(1), true);

            self.0.push_back(frames);
        }
    }

    fn split_node(&mut self, i: usize, num_of_pages: NumOfPages<Size4KiB>) {
        if self.0[i].available {
            while self.0[i].num_of_pages > num_of_pages {
                let start = self.0[i].start;
                let num_of_pages = self.0[i].num_of_pages;

                let new_frames = Frames::new(
                    start + num_of_pages.as_bytes().as_usize() / 2,
                    num_of_pages / 2,
                    true,
                );

                self.0[i].num_of_pages /= 2;
                self.0.insert(i + 1, new_frames);
            }
        }
    }

    fn merge_all_nodes(&mut self) {
        // By reversing the range, bit chunks of memory will go to the back of list.
        // This will make it faster to search a small amount of memory.
        for i in (0..self.0.len()).rev() {
            if self.mergeable(i) {
                self.merge_two_nodes(i);
                return self.merge_all_nodes();
            }
        }
    }

    fn mergeable(&self, i: usize) -> bool {
        if i < self.0.len() - 1 {
            let node = &self.0[i];
            let next = &self.0[i + 1];

            Self::two_nodes_available(node, next)
                && Self::two_nodes_consecutive(node, next)
                && Self::two_nodes_have_same_pages(node, next)
        } else {
            false
        }
    }

    fn two_nodes_available(node: &Frames, next: &Frames) -> bool {
        node.available && next.available
    }

    fn two_nodes_consecutive(node: &Frames, next: &Frames) -> bool {
        node.start + u64::try_from(node.num_of_pages.as_bytes().as_usize()).unwrap() == next.start
    }

    fn two_nodes_have_same_pages(node: &Frames, next: &Frames) -> bool {
        node.num_of_pages == next.num_of_pages
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
    fn new(start: PhysAddr, num_of_pages: NumOfPages<Size4KiB>, available: bool) -> Self {
        Self {
            start,
            num_of_pages,
            available,
        }
    }
}
