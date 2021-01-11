// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
    collections::{self, woken_pid},
    stack_frame::StackFrame,
    Privilege, Process,
};
use crate::{mem::allocator::page_box::PageBox, tests, tss::TSS};
use core::convert::TryInto;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{PageSize, PhysFrame, Size4KiB},
    VirtAddr,
};

pub fn switch() -> VirtAddr {
    if cfg!(feature = "qemu_test") {
        tests::process::count_switch();
    }

    change_current_process();
    switch_pml4();
    prepare_stack();
    register_current_stack_frame_with_tss();
    current_stack_frame_top_addr()
}

fn change_current_process() {
    woken_pid::change_active_pid();
}

fn switch_pml4() {
    let (_, f) = Cr3::read();
    let a = collections::process::handle_running(|p| p.pml4_addr);
    let a = PhysFrame::from_start_address(a).expect("PML4 is not aligned properly");

    // SAFETY: The PML4 frame is correct one and flags are unchanged.
    unsafe { Cr3::write(a, f) }
}

fn prepare_stack() {
    collections::process::handle_running_mut(|p| {
        if p.stack_frame.is_none() {
            StackCreator::new(p).create()
        }
    })
}

fn register_current_stack_frame_with_tss() {
    TSS.lock().interrupt_stack_table[0] = current_stack_frame_bottom_addr();
}

fn current_stack_frame_top_addr() -> VirtAddr {
    collections::process::handle_running(Process::stack_frame_top_addr)
}

fn current_stack_frame_bottom_addr() -> VirtAddr {
    collections::process::handle_running(Process::stack_frame_bottom_addr)
}

struct StackCreator<'a> {
    process: &'a mut Process,
}
impl<'a> StackCreator<'a> {
    fn new(process: &'a mut Process) -> Self {
        assert!(process.stack.is_none(), "Stack is already created.");
        assert!(
            process.stack_frame.is_none(),
            "Stack frame is already created."
        );

        Self { process }
    }

    fn create(mut self) {
        self.create_stack();
        self.create_stack_frame();
    }

    fn create_stack(&mut self) {
        assert!(self.process.stack.is_none(), "Stack is already created.");

        let stack = PageBox::kernel_slice(0, (Size4KiB::SIZE * 5).try_into().unwrap());
        self.process.stack = Some(stack);
    }

    fn create_stack_frame(&mut self) {
        assert!(
            self.process.stack_frame.is_none(),
            "Stack frame is already created."
        );

        match self.process.stack {
            Some(ref s) => {
                let stack_bottom = s.virt_addr() + s.bytes().as_usize();
                let stack_frame = PageBox::kernel(match self.process.privilege {
                    Privilege::Kernel => StackFrame::kernel(self.process.f, stack_bottom),
                    Privilege::User => StackFrame::user(self.process.f, stack_bottom),
                });

                self.process.stack_frame = Some(stack_frame);
            }
            None => panic!("Stack is not created."),
        }
    }
}
