    .text
    .code64
    .intel_syntax noprefix

    .set INTERRUPT_STACK, 0xffffffffc0000000 - (0x1000 * 16 / 2)

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

    .global prepare_exit_syscall
prepare_exit_syscall:
    mov rsp, INTERRUPT_STACK
    call exit_process
