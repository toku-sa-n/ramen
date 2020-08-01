    [bits 64]

    global memset
    global memcpy

    ; void *memset(void *dest, int c, size_t n);
    ; size_t == usize
memset:
    mov rax, rsi
    mov rcx, rdx
    rep stosb
    mov rax, rdi
    ret

    ; void *memcpy(void *dest, const void *src, size_t n);
memcpy:
    mov rcx, rdx
    rep movsb
    mov rax, rdi
    ret

