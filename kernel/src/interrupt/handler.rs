use {
    crate::{interrupt::apic::local, process},
    core::arch::asm,
};

#[no_mangle]
extern "C" fn h_20() -> u64 {
    local::end_of_interrupt();
    process::switch().as_u64()
}

#[no_mangle]
extern "C" fn h_80() -> u64 {
    let v = syscall_prepare_arguments();
    process::scheduler::assign_to_rax(v);
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
