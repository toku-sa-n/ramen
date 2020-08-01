    [bits 64]

    extern memset

    [section .text]

    ; void *memset(void *dest, int c, size_t n);
    ; size_t == usize
memset:
    mov rax, rsi
    mov rcx, rdx
    rep stosb
    ret
