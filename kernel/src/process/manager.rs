// SPDX-License-Identifier: GPL-3.0-or-later

use super::{stack_frame::StackFrame, Privilege, Process};
use crate::{mem::allocator::page_box::PageBox, tests, tss::TSS};
use alloc::collections::{BTreeMap, VecDeque};
use conquer_once::spin::Lazy;
use core::convert::TryInto;
use spinning_top::Spinlock;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{PageSize, PhysFrame, Size4KiB},
    VirtAddr,
};

static MANAGER: Lazy<Spinlock<Manager>> = Lazy::new(|| Spinlock::new(Manager::new()));

pub(super) fn main() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn add(p: Process) {
    MANAGER.lock().add(p);
}

pub fn switch() -> VirtAddr {
    MANAGER.lock().switch()
}

pub(super) fn getpid() -> i32 {
    MANAGER.lock().getpid()
}

struct Manager {
    pids: VecDeque<super::Id>,
    processes: BTreeMap<super::Id, Process>,
}
impl Manager {
    fn new() -> Self {
        Self {
            pids: VecDeque::new(),
            processes: BTreeMap::new(),
        }
    }

    fn add(&mut self, p: Process) {
        self.add_pid(p.id());
        self.add_process(p);
    }

    fn add_pid(&mut self, id: super::Id) {
        self.pids.push_back(id);
    }

    fn add_process(&mut self, p: Process) {
        let id = p.id();
        self.processes.insert(id, p);
    }

    fn switch(&mut self) -> VirtAddr {
        self.change_current_process();
        self.switch_pml4();
        self.prepare_stack();
        self.register_current_stack_frame_with_tss();
        self.current_stack_frame_top_addr()
    }

    fn getpid(&self) -> i32 {
        self.current_process().id.as_i32()
    }

    fn change_current_process(&mut self) {
        self.pids.rotate_left(1);
    }

    fn switch_pml4(&self) {
        let (_, f) = Cr3::read();
        let a = self.current_process().pml4_addr;
        let a = PhysFrame::from_start_address(a).expect("PML4 is not aligned properly");

        // SAFETY: The PML4 frame is correct one and flags are unchanged.
        unsafe { Cr3::write(a, f) }
    }

    fn prepare_stack(&mut self) {
        if self.current_process().stack_frame.is_none() {
            StackCreator::new(self.current_process_mut()).create();
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
        let id = self.current_pid();
        self.processes.get(&id).unwrap_or_else(|| {
            panic!(
                "Process of PID {} is not added to process collection",
                id.as_i32()
            )
        })
    }

    fn current_process_mut(&mut self) -> &mut Process {
        let id = self.current_pid();
        self.processes.get_mut(&id).unwrap_or_else(|| {
            panic!(
                "Process of PID {} id not added to process collection",
                id.as_i32()
            )
        })
    }

    fn current_pid(&self) -> super::Id {
        self.pids[0]
    }
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
                let instruction_pointer =
                    VirtAddr::new((self.process.f as usize).try_into().unwrap());
                let stack_bottom = s.virt_addr() + s.bytes().as_usize();

                let stack_frame = PageBox::kernel(match self.process.privilege {
                    Privilege::Kernel => StackFrame::kernel(instruction_pointer, stack_bottom),
                    Privilege::User => StackFrame::user(instruction_pointer, stack_bottom),
                });

                self.process.stack_frame = Some(stack_frame);
            }
            None => panic!("Stack is not created."),
        }
    }
}
