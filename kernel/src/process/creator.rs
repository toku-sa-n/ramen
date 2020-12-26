// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    VirtAddr,
};

use crate::mem::allocator::page_box::PageBox;

use super::{stack_frame::StackFrame, Process};

pub struct Creator {
    f: fn() -> !,
}
impl Creator {
    pub fn new(f: fn() -> !) -> Self {
        Self { f }
    }

    pub fn create(self) -> Process {
        let stack = PageBox::new_slice(0, Size4KiB::SIZE.try_into().unwrap());
        let stack_bottom_addr = stack.virt_addr() + stack.bytes().as_usize();
        let rip = VirtAddr::new((self.f as usize).try_into().unwrap());

        Process {
            _stack: stack,
            stack_frame: PageBox::new(StackFrame::new(rip, stack_bottom_addr)),
        }
    }
}
