    .text
    .code64
    .intel_syntax noprefix

    .global cause_timer_interrupt_asm
cause_timer_interrupt_asm:
    int 0x20

    .global syscall_prepare_arguments
    .extern select_proper_syscall
syscall_prepare_arguments:
    mov rcx, rdx
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, rax

    call select_proper_syscall

    ret
