// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use crate::gdt::GDT;
use rflags::RFlags;
use x86_64::{
    registers::rflags,
    structures::{gdt::SegmentSelector, idt::InterruptStackFrameValue},
    VirtAddr,
};

#[repr(C)]
#[derive(Debug)]
pub struct StackFrame {
    regs: GeneralRegisters,
    interrupt: InterruptStackFrameValue,
}
impl StackFrame {
    pub(super) fn new(f: fn(), stack_pointer: VirtAddr) -> Self {
        let cpu_flags = (rflags::read() | RFlags::INTERRUPT_FLAG).bits();
        let instruction_pointer =
            VirtAddr::new((super::manager::loader as usize).try_into().unwrap());
        let segs = Selectors::new();

        Self {
            regs: GeneralRegisters::new(f),
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
    fn new() -> Self {
        Self {
            code: GDT.kernel_code,
            data: GDT.kernel_data,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug)]
struct GeneralRegisters {
    _rax: u64,
    _rbx: u64,
    _rcx: u64,
    _rdx: u64,
    _rsi: u64,
    rdi: u64,
    _r8: u64,
    _r9: u64,
    _r10: u64,
    _r11: u64,
    _r12: u64,
    _r13: u64,
    _r14: u64,
    _r15: u64,
    _rbp: u64,
}
impl GeneralRegisters {
    fn new(f: fn()) -> Self {
        Self {
            rdi: (f as usize).try_into().unwrap(),
            ..Self::default()
        }
    }
}
