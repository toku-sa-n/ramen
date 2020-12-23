// SPDX-License-Identifier: GPL-3.0-or-later

mod manager;

use core::convert::TryInto;

use crate::{gdt::GDT, mem::allocator::page_box::PageBox, syscall};
use manager::{Manager, MANAGER};
use x86_64::{
    registers::rflags,
    structures::{
        idt::InterruptStackFrameValue,
        paging::{PageSize, Size4KiB},
    },
    VirtAddr,
};

pub fn init() {
    let mut m = MANAGER.lock();

    let pa = Process::new(task_a);
    let pb = Process::new(task_b);

    m.add_process(pa);
    m.add_process(pb);
}

fn task_a() {
    info!("Task A");
    loop {
        syscall::enable_interrupt_and_halt();
    }
}

fn task_b() {
    info!("Task B");
    loop {
        syscall::enable_interrupt_and_halt();
    }
}

pub struct Process {
    _stack: PageBox<[u8]>,
    stack_frame: PageBox<StackFrame>,
}
impl Process {
    fn new(f: fn()) -> Self {
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
        self.stack_frame.virt_addr() + self.stack_frame.bytes().as_usize()
    }
}

pub fn switch() -> VirtAddr {
    Manager::switch_process()
}

#[repr(C)]
struct StackFrame {
    regs: GeneralRegisters,
    _err_code: u64,
    interrupt: InterruptStackFrameValue,
}
impl StackFrame {
    fn new(instruction_pointer: VirtAddr, stack_pointer: VirtAddr) -> Self {
        Self {
            regs: GeneralRegisters::default(),
            _err_code: 0,
            interrupt: InterruptStackFrameValue {
                instruction_pointer,
                code_segment: GDT.user_code.0.into(),
                cpu_flags: rflags::read().bits(),
                stack_pointer,
                stack_segment: GDT.user_data.0.into(),
            },
        }
    }
}

#[repr(C)]
#[derive(Default)]
struct GeneralRegisters {
    _rax: u64,
    _rbx: u64,
    _rcx: u64,
    _rdx: u64,
    _rsi: u64,
    _rdi: u64,
    _r8: u64,
    _r9: u64,
    _r10: u64,
    _r11: u64,
    _r12: u64,
    _r13: u64,
    _r14: u64,
    _r15: u64,
}
