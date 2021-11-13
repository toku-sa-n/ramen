// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use {
    crate::gdt::SELECTORS,
    rflags::RFlags,
    x86_64::{
        registers::rflags,
        structures::{gdt::SegmentSelector, idt::InterruptStackFrameValue},
        VirtAddr,
    },
};

#[repr(C)]
#[derive(Debug)]
pub(super) struct StackFrame {
    pub(super) regs: GeneralRegisters,
    interrupt: InterruptStackFrameValue,
}
impl StackFrame {
    pub(crate) fn kernel(entry: VirtAddr, stack_pointer: VirtAddr) -> Self {
        Self::new(entry, stack_pointer, &Selectors::kernel())
    }

    pub(crate) fn user(entry: VirtAddr, stack_pointer: VirtAddr) -> Self {
        Self::new(entry, stack_pointer, &Selectors::user())
    }

    fn new(entry: VirtAddr, stack_pointer: VirtAddr, segs: &Selectors) -> Self {
        let cpu_flags = (rflags::read() | RFlags::INTERRUPT_FLAG).bits();
        let instruction_pointer = VirtAddr::new((super::loader as usize).try_into().unwrap());

        Self {
            regs: GeneralRegisters::new(entry),
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
        Self::new(SELECTORS.kernel_code, SELECTORS.kernel_data)
    }

    fn user() -> Self {
        Self::new(SELECTORS.user_code, SELECTORS.user_data)
    }

    fn new(code: SegmentSelector, user: SegmentSelector) -> Self {
        Self { code, data: user }
    }
}

#[repr(C)]
#[derive(Default, Debug)]
pub(super) struct GeneralRegisters {
    pub(super) rax: u64,
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
    fn new(entry: VirtAddr) -> Self {
        Self {
            rdi: entry.as_u64(),
            ..Self::default()
        }
    }
}
