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
