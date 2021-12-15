use {
    crate::{interrupt::apic::local, process},
    core::arch::asm,
    x86_64::structures::idt::InterruptStackFrame,
};

pub(super) extern "x86-interrupt" fn h_20(_: InterruptStackFrame) {
    local::end_of_interrupt();
    process::switch();
}

pub(super) extern "x86-interrupt" fn h_80(_: InterruptStackFrame) {
    let v = syscall_prepare_arguments();
    local::end_of_interrupt();
    process::switch();
}

pub(super) extern "x86-interrupt" fn h_81(_: InterruptStackFrame) {
    syscall_prepare_arguments();
    local::end_of_interrupt();
    process::switch();
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
