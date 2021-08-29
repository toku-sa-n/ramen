use crate::gdt::GDT;
use common::constant::KERNEL_ADDR;
use core::mem::size_of;
use static_assertions::const_assert_eq;
use x86_64::{registers::rflags::RFlags, structures::paging::PhysFrame, VirtAddr};

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(super) struct Context {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,

    rsp: u64,
    rbp: u64,
    rsi: u64,
    rdi: u64,

    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,

    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,

    cs: u64,
    ss: u64,
    fs: u64,
    gs: u64,

    cr3: u64,
    rip: u64,
    rflags: u64,
    _fxsave_requires_16_bytes_alinged_address: u64,

    fxsave_area: [u128; 4],
}
const_assert_eq!(size_of::<Context>(), 8 * 4 * 6 + 16 * 4);
impl Context {
    pub(super) fn user(rip: VirtAddr, pml4: PhysFrame) -> Self {
        Self {
            rsp: KERNEL_ADDR.as_u64(),
            rip: rip.as_u64(),
            rflags: (RFlags::INTERRUPT_FLAG | RFlags::PARITY_FLAG).bits(),
            cr3: pml4.start_address().as_u64(),
            cs: GDT.user_code.0.into(),
            ss: GDT.user_data.0.into(),
            fs: GDT.user_data.0.into(),
            gs: GDT.user_data.0.into(),
            ..Self::default()
        }
    }

    pub(super) fn kernel(rip: VirtAddr, pml4: PhysFrame) -> Self {
        Self {
            rsp: KERNEL_ADDR.as_u64(),
            rip: rip.as_u64(),
            rflags: (RFlags::INTERRUPT_FLAG | RFlags::PARITY_FLAG).bits(),
            cr3: pml4.start_address().as_u64(),
            cs: GDT.kernel_code.0.into(),
            ss: GDT.kernel_data.0.into(),
            fs: GDT.kernel_data.0.into(),
            gs: GDT.kernel_data.0.into(),
            ..Self::default()
        }
    }

    #[allow(clippy::too_many_lines)]
    #[naked]
    extern "sysv64" fn switch_context(old_context: *mut Self, new_context: *mut Self) {
        unsafe {
            asm!(
                "
            mov [rdi+0x00], rax
            mov [rdi+0x08], rbx
            mov [rdi+0x10], rcx
            mov [rdi+0x18], rdx

            lea rax, [rsp+0x08]
            mov [rdi+0x20], rax
            mov [rdi+0x28], rbp
            mov [rdi+0x30], r8
            mov [rdi+0x38], r9
            mov [rdi+0x40], r10
            mov [rdi+0x48], r11
            mov [rdi+0x50], r12
            mov [rdi+0x58], r13
            mov [rdi+0x60], r14
            mov [rdi+0x68], r15

            mov rax, cr3
            mov [rdi+0x70], rax
            mov [rdi+0x78], cs
            mov [rdi+0x80], ss
            mov [rdi+0x88], fs
            mov [rdi+0x90], gs

            mov rax, [rsp]
            mov [rdi+0x98], rax
            pushfq
            pop qword ptr [rdi+0x100]

            fxsave [rdi+0x110]

            mov rax, [rsi+0x70]
            mov cr3, rax

            mov rax, [rsi+0x88]
            mov fs, ax

            mov rax, [rsi+0x90]
            mov gs, ax

            mov rax, [rsi+0x00]
            mov rbx, [rsi+0x08]
            mov rcx, [rsi+0x10]
            mov rdx, [rsi+0x18]
            mov rbp, [rsi+0x28]
            mov r8, [rsi+0x30]
            mov r9, [rsi+0x38]
            mov r10, [rsi+0x40]
            mov r11, [rsi+0x48]
            mov r12, [rsi+0x50]
            mov r13, [rsi+0x58]
            mov r14, [rsi+0x60]
            mov r15, [rsi+0x68]

            fxrstor [rsi+0x110]

            push qword ptr [rsi+0x80]
            push qword ptr [rsi+0x20]
            push qword ptr [rsi+0x100]
            push qword ptr [rsi+0x78]
            push qword ptr [rsi+0x98]

            iretq
            ",
                options(noreturn),
            );
        }
    }
}
