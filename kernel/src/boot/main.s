// SPDX-License-Identifier: GPL-3.0-or-later

    .section .boot, "ax"
    .intel_syntax noprefix
    .global _start

    .extern os_main
    .extern KERNEL_PML4
    .extern MULTIBOOT2_SIGNATURE_LMA
    .extern BOOT_INFO_ADDR_LMA

    .code32

_start:

    // Save the Multiboot2 informations
    lea edx, [MULTIBOOT2_SIGNATURE_LMA]
    mov [edx], eax

    lea edx, [BOOT_INFO_ADDR_LMA]
    mov [edx], ebx

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

loop:
    jmp loop
