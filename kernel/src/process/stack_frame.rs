// SPDX-License-Identifier: GPL-3.0-or-later

use rflags::RFlags;
use x86_64::{registers::rflags, structures::idt::InterruptStackFrameValue, VirtAddr};

use crate::gdt::GDT;

#[repr(C)]
pub struct StackFrame {
    regs: GeneralRegisters,
    interrupt: InterruptStackFrameValue,
}
impl StackFrame {
    pub fn user(instruction_pointer: VirtAddr, stack_pointer: VirtAddr) -> Self {
        let cpu_flags = (rflags::read() | RFlags::INTERRUPT_FLAG).bits();
        Self {
            regs: GeneralRegisters::default(),
            interrupt: InterruptStackFrameValue {
                instruction_pointer,
                code_segment: GDT.user_code.0.into(),
                cpu_flags,
                stack_pointer,
                stack_segment: GDT.user_data.0.into(),
            },
        }
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
