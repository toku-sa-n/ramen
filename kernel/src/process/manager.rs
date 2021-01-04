// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use crate::{
    mem::{allocator::page_box::PageBox, paging::pml4::PML4},
    tss::TSS,
};

use super::{stack_frame::StackFrame, Process};
use alloc::collections::VecDeque;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{PageSize, PageTable, PageTableFlags, PhysFrame, Size4KiB},
    VirtAddr,
};

static MANAGER: Lazy<Spinlock<Manager>> = Lazy::new(|| Spinlock::new(Manager::new()));

pub fn add_process(p: Process) {
    MANAGER.lock().add_process(p);
}

pub fn switch_process() -> VirtAddr {
    MANAGER.lock().switch_process()
}

pub fn exit() -> ! {
    MANAGER.lock().exit();
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
        self.create_pml4_if_not_created();
        self.switch_pml4();
        self.prepare_stack();
        self.register_current_stack_frame_with_tss();
        self.current_stack_frame_top_addr()
    }

    fn exit(&mut self) -> ! {
        self.mark_current_process_as_exit();
        Self::make_timer_interrupt();
    }

    fn change_current_process(&mut self) {
        loop {
            self.processes.rotate_left(1);
            if self.current_process().running {
                break;
            }

            self.processes.pop_front();
        }
    }

    fn create_pml4_if_not_created(&mut self) {
        if !self.pml4_exists() {
            self.create_pml4();
        }
    }

    fn pml4_exists(&self) -> bool {
        self.current_process().pml4.is_some()
    }

    fn create_pml4(&mut self) {
        let proc = self.current_process_mut();
        assert!(proc.pml4.is_none(), "PML4 is already created.");

        let p = Pml4Creator::new().create();
        let a = p.phys_addr();

        self.current_process_mut().pml4 = Some(p);
        self.current_process_mut().pml4_addr = Some(a);
    }

    fn switch_pml4(&self) {
        let (_, f) = Cr3::read();
        let a = self
            .current_process()
            .pml4_addr
            .expect("PML4 is not created.");
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

    fn mark_current_process_as_exit(&mut self) {
        self.current_process_mut().running = false;
    }

    fn make_timer_interrupt() -> ! {
        unsafe {
            asm!("int 0x20", options(noreturn));
        }
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
                let stack_frame = PageBox::kernel(StackFrame::new(&self.process.f, stack_bottom));

                self.process.stack_frame = Some(stack_frame);
            }
            None => panic!("Stack is not created."),
        }
    }
}

pub struct Pml4Creator {
    pml4: PageBox<PageTable>,
}
impl Pml4Creator {
    pub fn new() -> Self {
        Self {
            pml4: PageBox::kernel(PageTable::new()),
        }
    }

    pub fn create(mut self) -> PageBox<PageTable> {
        self.enable_recursive_paging();
        self.map_kernel_area();
        self.pml4
    }

    fn enable_recursive_paging(&mut self) {
        let a = self.pml4.phys_addr();
        let f =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        self.pml4[511].set_addr(a, f);
    }

    fn map_kernel_area(&mut self) {
        self.pml4[510] = PML4.lock().level_4_table()[510].clone();
    }
}
