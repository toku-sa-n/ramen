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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    fxsave_area: [u8; 512],
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

    pub(super) fn switch(old: *mut Context, new: *mut Context) {
        extern "sysv64" {
            fn asm_switch_context(old: *mut Context, new: *mut Context);
        }

        unsafe {
            asm_switch_context(old, new);
        }
    }

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
impl Default for Context {
    fn default() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsp: 0,
            rbp: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            cs: 0,
            ss: 0,
            fs: 0,
            gs: 0,
            cr3: 0,
            rip: 0,
            rflags: 0,
            _fxsave_requires_16_bytes_alinged_address: 0,
            fxsave_area: [0; 512],
        }
    }
}
