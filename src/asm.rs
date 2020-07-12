pub fn hlt() -> () {
    unsafe {
        asm!("hlt");
    }
}

pub fn sti() -> () {
    unsafe {
        asm!("sti");
    }
}

pub fn stihlt() -> () {
    unsafe {
        asm!(
            "sti
             hlt"
        );
    }
}

pub fn cli() -> () {
    unsafe {
        asm!("cli");
    }
}

pub fn out8(port: u32, data: u8) -> () {
    unsafe {
        asm!("out dx, al",in("dx") port,in("al") data);
    }
}

// It might be true that the first line can be deleted because the lower bits of EDX are DX
// itself.
pub fn in8(port: u32) -> u8 {
    let result: u8;
    unsafe {
        asm!("mov edx, {:e}",in(reg) port);
        asm!("mov eax, 0");
        asm!("in al, dx", out("al") result);
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

pub fn load_interrupt_descriptor_table_register(limit: u32, address: u64) {
    unsafe {
        asm!("lidt [{:r}]",in(reg) &GdtrIdtrData::new(limit as i16, address));
    }
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
                    "
                );
                asm!("call {}",in(reg) ($function_name as extern "C" fn()->()));
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
                    iretq"
                );
            }
        }
        handler_wrapper
    }}
}
