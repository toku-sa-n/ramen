    .text
    .code64
    .intel_syntax noprefix

    .global cause_timer_interrupt_asm
cause_timer_interrupt_asm:
    int 0x20
