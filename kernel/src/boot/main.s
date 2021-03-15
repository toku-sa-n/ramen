// SPDX-License-Identifier: GPL-3.0-or-later

    .section .boot, "ax"
    .intel_syntax noprefix
    .global _start

    .extern KERNEL_PML4_LMA
    .extern KERNEL_PDPT_LMA
    .extern KERNEL_LMA

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
    or eax, 1   /* Present */
    mov [edx], eax

    // PDPT.
    lea edx, [KERNEL_PDPT_LMA]
    mov eax, 0x81  /* 1G, Present */
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

loop:
    jmp loop
