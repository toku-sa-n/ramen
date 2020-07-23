pub fn hlt() -> () {
    unsafe {
        asm!("hlt", options(nomem, preserves_flags, nostack));
    }
}

pub fn sti() -> () {
    unsafe {
        asm!("sti", options(nomem, preserves_flags, nostack));
    }
}

pub fn stihlt() -> () {
    unsafe {
        asm!(
            "sti
             hlt",
            options(nomem, preserves_flags, nostack)
        );
    }
}

pub fn cli() -> () {
    unsafe {
        asm!("cli", options(nomem, preserves_flags, nostack));
    }
}

pub fn out8(port: u32, data: u8) -> () {
    unsafe {
        asm!("out dx, al",in("dx") port,in("al") data,options(nomem, preserves_flags, nostack));
    }
}

// It might be true that the first line can be deleted because the lower bits of EDX are DX
// itself.
pub fn in8(port: u32) -> u8 {
    let result: u8;
    unsafe {
        asm!("mov edx, {:e}",in(reg) port,options(nomem, preserves_flags, nostack));
        asm!("mov eax, 0", options(nomem, preserves_flags, nostack));
        asm!("in al, dx", out("al") result,options(nomem, preserves_flags, nostack));
    }
    result
}

#[repr(C, packed)]
struct GdtrIdtrData {
    _limit: i16,
    _address: u64,
}

impl GdtrIdtrData {
    fn new(limit: i16, address: u64) -> Self {
        Self {
            _limit: limit,
            _address: address,
        }
    }
}

pub fn lidt(limit: u16, address: u64) {
    unsafe {
        asm!("lidt [{:r}]",in(reg) &GdtrIdtrData::new(limit as i16, address),options(readonly, preserves_flags, nostack));
    }
}

pub fn lgdt(limit: u16, address: u64) {
    unsafe {
        asm!("lgdt [{:r}]",in(reg) &GdtrIdtrData::new(limit as i16, address),options(readonly, preserves_flags, nostack));
    }
}

/// Safety: `offset_of_cs` must be a valid offset to code segment. Otherwise unexpected
/// behavior will occur.
pub unsafe fn set_code_segment(offset_of_cs: u16) {
    asm!("push {0:r}
    lea rax, 1f
    push rax
    retfq
    1:", in(reg) offset_of_cs,options(preserves_flags));
}

/// Safety: `offset_of_ds` must be a valid offset to data segment. Otherwise unexpected
/// behavior will occur.
pub unsafe fn set_data_segment(offset_of_ds: u16) {
    asm!("mov es, ax
    mov ss, ax
    mov ds, ax
    mov fs, ax
    mov gs, ax",in("ax") offset_of_ds,options(nomem, preserves_flags, nostack));
}

// Don't put these asm! in one! It doesn't work!
#[macro_export]
macro_rules! interrupt_handler{
    ($function_name:ident)=>{{
        #[naked]
        pub extern "C" fn handler_wrapper() -> () {
            // In 64-bit mode, ES, DS, and SS segment registers are not used.
            // It's not necessary to push these registers.
            unsafe{
                asm!("
                    push rax
                    push rcx
                    push rdx
                    push rbx
                    push rsp
                    push rbp
                    push rsi
                    push rdi
                    push r8
                    push r9
                    push r10
                    push r11
                    push r12
                    push r13
                    push r14
                    push r15
                    ",options(preserves_flags)
                );
                asm!("call {}",in(reg) ($function_name as extern "C" fn()->()),options(preserves_flags));
                asm!("
                    pop r15
                    pop r14
                    pop r13
                    pop r12
                    pop r11
                    pop r10
                    pop r9
                    pop r8
                    pop rdi
                    pop rsi
                    pop rbp
                    pop rsp
                    pop rbx
                    pop rdx
                    pop rcx
                    pop rax
                    iretq",options(preserves_flags)
                );
            }
        }
        handler_wrapper
    }}
}
