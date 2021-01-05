// SPDX-License-Identifier: GPL-3.0-or-later

use crate::gdt::GDT;
use rflags::RFlags;
use x86_64::{
    registers::rflags,
    structures::{gdt::SegmentSelector, idt::InterruptStackFrameValue},
    VirtAddr,
};

#[repr(C)]
pub struct StackFrame {
    regs: GeneralRegisters,
    interrupt: InterruptStackFrameValue,
}
impl StackFrame {
    pub fn kernel(instruction_pointer: VirtAddr, stack_pointer: VirtAddr) -> Self {
        Self::new(instruction_pointer, stack_pointer, Selectors::kernel())
    }

    pub fn user(instruction_pointer: VirtAddr, stack_pointer: VirtAddr) -> Self {
        Self::new(instruction_pointer, stack_pointer, Selectors::user())
    }

    fn new(instruction_pointer: VirtAddr, stack_pointer: VirtAddr, segs: Selectors) -> Self {
        let cpu_flags = (rflags::read() | RFlags::INTERRUPT_FLAG).bits();
        Self {
            regs: GeneralRegisters::default(),
            interrupt: InterruptStackFrameValue {
                instruction_pointer,
                code_segment: segs.code.0.into(),
                cpu_flags,
                stack_pointer,
                stack_segment: segs.data.0.into(),
            },
        }
    }
}

struct Selectors {
    code: SegmentSelector,
    data: SegmentSelector,
}
impl Selectors {
    fn kernel() -> Self {
        Self::new(GDT.kernel_code, GDT.kernel_data)
    }

    fn user() -> Self {
        Self::new(GDT.user_code, GDT.user_data)
    }

    fn new(code: SegmentSelector, user: SegmentSelector) -> Self {
        Self { code, data: user }
    }
}

#[repr(C)]
#[derive(Default)]
struct GeneralRegisters {
    _rbp: u64,
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
