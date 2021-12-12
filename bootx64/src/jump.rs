use {predefined_mmap::STACK_BASE, x86_64::VirtAddr};

pub fn to_kernel(mut boot_info: boot_info::Info) -> ! {
    disable_interruption();

    switch_stack_and_call_kernel_code(&mut boot_info, boot_info.entry_addr(), STACK_BASE)
}

fn disable_interruption() {
    // Use `nop` because some machines go wrong when continuously doing `out`.
    unsafe {
        asm!(
            "mov al,0xff
            out 0x21,al
            nop
            out 0xa1,al
            cli"
        );
    }
}

#[naked]
extern "sysv64" fn switch_stack_and_call_kernel_code(
    boot_info: *mut boot_info::Info,
    entry: VirtAddr,
    stack_ptr: VirtAddr,
) -> ! {
    unsafe {
        asm!(
            "
        mov rsp, rdx
        jmp rsi
            ",
            options(noreturn)
        );
    }
}
