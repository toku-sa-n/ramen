// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use crate::{mem::allocator::page_box::PageBox, tss::TSS};

use super::{stack_frame::StackFrame, Process};
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{PageSize, PhysFrame, Size4KiB},
    VirtAddr,
};

static MANAGER: Lazy<Spinlock<Manager>> = Lazy::new(|| Spinlock::new(Manager::new()));

pub fn add_process(p: Process) {
    MANAGER.lock().add_process(p);
}

pub fn switch_process() -> VirtAddr {
    MANAGER.lock().switch_process()
}

struct Manager {
    processes: VecDeque<Process>,
}
impl Manager {
    fn new() -> Self {
        Self {
            processes: VecDeque::new(),
        }
    }

    fn add_process(&mut self, p: Process) {
        self.processes.push_back(p)
    }

    fn switch_process(&mut self) -> VirtAddr {
        self.change_current_process();
        self.switch_pml4();
        self.prepare_stack();
        self.register_current_stack_frame_with_tss();
        self.current_stack_frame_top_addr()
    }

    fn change_current_process(&mut self) {
        self.processes.rotate_left(1);
    }

    fn switch_pml4(&self) {
        let (_, f) = Cr3::read();
        let a = self.current_process().pml4_addr;
        let a = PhysFrame::from_start_address(a).expect("PML4 is not aligned properly");

        // SAFETY: The PML4 frame is correct one and flags are unchanged.
        unsafe { Cr3::write(a, f) }
    }

    fn prepare_stack(&mut self) {
        if let None = self.current_process().stack_frame {
            self.create_stack();
            self.create_stack_frame();
        }
    }

    fn create_stack(&mut self) {
        let p = self.current_process_mut();

        assert!(p._stack.is_none(), "Stack is already created.");

        let stack = PageBox::kernel_slice(0, (Size4KiB::SIZE * 5).try_into().unwrap());
        p._stack = Some(stack);
    }

    fn create_stack_frame(&mut self) {
        let p = self.current_process_mut();

        assert!(p.stack_frame.is_none(), "Stack frame is already created.");

        match p._stack {
            Some(ref s) => {
                let rip = VirtAddr::new((p.f as usize).try_into().unwrap());
                let rsp = s.virt_addr() + s.bytes().as_usize();

                let stack_frame = PageBox::kernel(StackFrame::new(rip, rsp));

                p.stack_frame = Some(stack_frame);
            }
            None => panic!("Stack is not created."),
        }
    }

    fn register_current_stack_frame_with_tss(&mut self) {
        TSS.lock().interrupt_stack_table[0] = self.current_stack_frame_bottom_addr();
    }

    fn current_stack_frame_top_addr(&self) -> VirtAddr {
        self.current_process().stack_frame_top_addr()
    }

    fn current_stack_frame_bottom_addr(&self) -> VirtAddr {
        self.current_process().stack_frame_bottom_addr()
    }

    fn current_process(&self) -> &Process {
        &self.processes[0]
    }

    fn current_process_mut(&mut self) -> &mut Process {
        &mut self.processes[0]
    }
}
