use {
    crate::gdt,
    core::mem::size_of,
    static_assertions::const_assert_eq,
    x86_64::{
        registers::rflags::RFlags,
        structures::{gdt::SegmentSelector, paging::PhysFrame},
        VirtAddr,
    },
};

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    fxsave_area: FxsaveArea,
}
const_assert_eq!(size_of::<Context>(), 8 * 4 * 6 + 512);
impl Context {
    pub(super) fn kernel(entry: VirtAddr, pml4: PhysFrame, rsp: VirtAddr) -> Self {
        Self::new(
            entry,
            pml4,
            rsp,
            gdt::kernel_code_selector(),
            gdt::kernel_data_selector(),
        )
    }

    pub(super) fn user(entry: VirtAddr, pml4: PhysFrame, rsp: VirtAddr) -> Self {
        Self::new(
            entry,
            pml4,
            rsp,
            gdt::user_code_selector(),
            gdt::user_data_selector(),
        )
    }

    #[naked]
    #[allow(clippy::too_many_lines)]
    pub(super) extern "sysv64" fn switch(old: *mut Context, new: *mut Context) {
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
    mov [rdi+0x30], rsi
    mov [rdi+0x38], rdi

    mov [rdi+0x40], r8
    mov [rdi+0x48], r9
    mov [rdi+0x50], r10
    mov [rdi+0x58], r11

    mov [rdi+0x60], r12
    mov [rdi+0x68], r13
    mov [rdi+0x70], r14
    mov [rdi+0x78], r15

    mov [rdi+0x80], cs
    mov [rdi+0x88], ss
    mov [rdi+0x90], fs
    mov [rdi+0x98], gs

    mov rax, cr3
    mov [rdi+0xa0], rax
    mov rax, [rsp]
    mov [rdi+0xa8], rax
    pushfq
    pop qword ptr [rdi+0xb0]

    fxsave [rdi+0xc0]

    mov rax, [rsi+0x00]
    mov rbx, [rsi+0x08]
    mov rcx, [rsi+0x10]
    mov rdx, [rsi+0x18]

    mov rbp, [rsi+0x28]
    mov rdi, [rsi+0x38]

    mov r8, [rsi+0x40]
    mov r9, [rsi+0x48]
    mov r10, [rsi+0x50]
    mov r11, [rsi+0x58]

    mov r12, [rsi+0x60]
    mov r13, [rsi+0x68]
    mov r14, [rsi+0x70]
    mov r15, [rsi+0x78]

    mov rax, [rsi+0x90]
    mov fs, ax
    mov rax, [rsi+0x98]
    mov gs, ax

    mov rax, [rsi+0xa0]
    mov cr3, rax

    fxrstor [rsi+0xc0]

    push qword ptr [rsi+0x88]
    push qword ptr [rsi+0x20]
    push qword ptr [rsi+0xb0]
    push qword ptr [rsi+0x80]
    push qword ptr [rsi+0xa8]

    mov rsi, [rsi+0x30]

    iretq
    ",
                options(noreturn)
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        entry: VirtAddr,
        pml4: PhysFrame,
        rsp: VirtAddr,
        code_segment: SegmentSelector,
        data_segment: SegmentSelector,
    ) -> Self {
        assert_eq!(
            rsp.as_u64() % 16,
            8,
            "`RSP % 16` must be 8. We must simulate the condition after calling the main function."
        );

        Self {
            rsp: rsp.as_u64(),
            rip: entry.as_u64(),
            rflags: (RFlags::INTERRUPT_FLAG | RFlags::PARITY_FLAG).bits(),
            cr3: pml4.start_address().as_u64(),
            cs: code_segment.0.into(),
            ss: data_segment.0.into(),
            fs: data_segment.0.into(),
            gs: data_segment.0.into(),
            ..Self::default()
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct FxsaveArea([u8; 512]);
impl Default for FxsaveArea {
    fn default() -> Self {
        Self([0; 512])
    }
}
