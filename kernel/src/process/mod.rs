// SPDX-License-Identifier: GPL-3.0-or-later

mod manager;
mod stack_frame;

use crate::{mem::allocator::page_box::PageBox, tss::TSS};
use common::constant::INTERRUPT_STACK;
use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use stack_frame::StackFrame;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr, VirtAddr,
};

static KERNEL_PML4: OnceCell<PhysAddr> = OnceCell::uninit();

pub fn init() {
    register_initial_interrupt_stack_table_addr();
}

pub fn add(p: Process) {
    manager::add_process(p);
}

pub fn switch() -> VirtAddr {
    manager::switch_process()
}

fn register_initial_interrupt_stack_table_addr() {
    TSS.lock().interrupt_stack_table[0] = INTERRUPT_STACK;
}

pub struct Process {
    _stack: PageBox<[u8]>,
    stack_frame: PageBox<StackFrame>,
}
impl Process {
    pub fn new(f: fn() -> !) -> Self {
        let stack = PageBox::new_slice(0, Size4KiB::SIZE.try_into().unwrap());
        let stack_bottom_addr = stack.virt_addr() + stack.bytes().as_usize();
        let rip = VirtAddr::new((f as usize).try_into().unwrap());
        Self {
            _stack: stack,
            stack_frame: PageBox::new(StackFrame::new(rip, stack_bottom_addr)),
        }
    }

    fn stack_frame_top_addr(&self) -> VirtAddr {
        self.stack_frame.virt_addr()
    }

    fn stack_frame_bottom_addr(&self) -> VirtAddr {
        self.stack_frame_top_addr() + self.stack_frame.bytes().as_usize()
    }
}
