use crate::{interrupt::apic::local, process};

#[no_mangle]
extern "C" fn h_20() -> u64 {
    local::end_of_interrupt();
    process::switch().as_u64()
}

#[no_mangle]
extern "C" fn h_81() -> u64 {
    syscall_prepare_arguments();
    local::end_of_interrupt();
    process::switch().as_u64()
}

#[naked]
extern "sysv64" fn syscall_prepare_arguments() -> u64 {
    unsafe {
        asm!(
            "
    mov rcx, rdx
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, rax

    call select_proper_syscall

    ret",
            options(noreturn)
        )
    }
}
