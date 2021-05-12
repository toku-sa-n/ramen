    .text
    .code64
    .intel_syntax noprefix

    .global exit_qemu_as_success
exit_qemu_as_success:
    mov dx, 0xf4
    mov eax, 33 >> 1
    out dx, eax
    jmp infinite_loop

    .global exit_qemu_as_failure
exit_qemu_as_failure:
    mov dx, 0xf4
    mov eax, 1
    out dx, eax
    jmp infinite_loop

infinite_loop:
    hlt
    jmp infinite_loop
