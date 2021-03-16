// SPDX-License-Identifier: GPL-3.0-or-later

    .section .boot, "ax"
    .intel_syntax noprefix
    .global _start

    .extern KERNEL_PML4_LMA
    .extern KERNEL_PDPT_LMA
    .extern KERNEL_LMA

    .extern MULTIBOOT2_INFORMATION_LMA

    .extern KERNEL_STACK
    .extern KERNEL_STACK_LMA
    .extern os_main

    .set MULTIBOOT2_SIGNATURE, 0x36d76289
    .set MULTIBOOT2_MAX_SIZE, 0x100000

_start:
    .code32

    cmp eax, MULTIBOOT2_SIGNATURE
    jne invalid_multiboot2_signature

    mov ecx, [ebx]
    cmp ecx, MULTIBOOT2_MAX_SIZE
    jg multiboot2_information_too_big

    mov esi, ebx
    lea edi, [MULTIBOOT2_INFORMATION_LMA]
    rep movsb

    // Enter the Long mode.
    //
    // Disable paging.
    mov eax, cr0
    and eax, 0x7fffffff
    mov cr0, eax

    // Enable PAE.
    mov eax, cr4
    or eax, 0x20
    mov cr4, eax

    // Prepare page tables

    // Recursive paging
    lea edx, [KERNEL_PML4_LMA]
    mov eax, edx
    or eax, 3   /* Present, writable. */
    mov [edx + 511 * 8], eax
    mov dword ptr [edx + 511 * 8 + 4], 0

    // For the bootloader.

    // PML4
    lea eax, [KERNEL_PDPT_LMA]
    or eax, 3   /* Writable, present */
    mov [edx], eax

    // PDPT.
    lea edx, [KERNEL_PDPT_LMA]
    mov eax, 0x83  /* 1G, writable, present */
    mov [edx], eax

    // For the kernel

    // PDPT of the kernel mapping is the same table as PML4.
    lea edx, [KERNEL_PML4_LMA]

    lea eax, [KERNEL_LMA]
    or eax, 0x83   /* 1G, Present, writable. */
    mov [edx + 510 * 8], eax
    mov dword ptr [edx + 510 * 8 + 4], 0

    // Change PML4
    lea eax, [KERNEL_PML4_LMA]
    mov cr3, eax

    // Enter the Long mode.
    mov ecx, 0xc0000080
    rdmsr
    or eax, 0x100
    wrmsr

    // Enable Paging
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax

paging_enabled:
    .code64

    // Adjust the stack pointer.
    lea rax, KERNEL_STACK_LMA
    mov rsp, rax

    // Switch segments
    lgdt [lgdt_values]

    // Code Segment
    push 8
    lea rax, switch_cs
    pushq rax

    retfq

switch_cs:
    .code64

    // All the others
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    // Setup kernel stack
    lea rsp, [KERNEL_STACK]

    mov rdi, os_main
    jmp rdi

lgdt_values:
    .word segments_end - segments - 1
    .quad segments

    .align 8
segments:
    // Null
    .long 0
    .long 0
    // Code
    .long 0
    .byte 0
    .byte 0x9a
    .byte 0xa0
    .byte 0
segments_end:

    // TODO: Print a friendly message.
invalid_multiboot2_signature:
multiboot2_information_too_big:
    hlt
    jmp invalid_multiboot2_signature
