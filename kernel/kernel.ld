OUTPUT_FORMAT(elf64-x86-64);

ENTRY(_start)

EXTERN(multiboot_header)

MEMORY{
    boot (RX)        : ORIGIN = 0x100000,           LENGTH = 0x1000
    kernel (RWX)     : ORIGIN = 0xffffffff80000000, LENGTH = 0x200000
    kernel_lma (RWX) : ORIGIN = 0x101000,           LENGTH = 0x200000
}

KERNEL_LMA_OFFSET = 0xffffffff80000000 - 0x101000;

SECTIONS
{
    . = 0x100000;

    .multiboot ALIGN(8) : {
        KEEP(*(.multiboot*))
    } > boot

    .boot : {
        *(.boot*)
    } > boot

    .text : {
        KERNEL_LMA = . - KERNEL_LMA_OFFSET;

        *(.text*)
    } > kernel AT > kernel_lma

    .rodata : {
        *(.rodata*)
    } > kernel AT > kernel_lma

    .data : {
        *(.data*)
    } > kernel AT > kernel_lma

    .bss : {
        *(.bss*)

        HEAP_START = .;
        . += 0x40000;
        HEAP_END = .;

        . = ALIGN(4K);
        KERNEL_PML4_LMA = . - KERNEL_LMA_OFFSET;
        . += 4K;

        KERNEL_PDPT_LMA = . - KERNEL_LMA_OFFSET;
        . += 4K;

        KERNEL_PD_LMA = . - KERNEL_LMA_OFFSET;
        . += 4K;

        KERNEL_PT_LMA = . - KERNEL_LMA_OFFSET;
        . += 4K;

        . += 8K;
        . = ALIGN(16);
        KERNEL_STACK = .;
        KERNEL_STACK_LMA = . - KERNEL_LMA_OFFSET;

        MULTIBOOT2_INFORMATION_LMA = . - KERNEL_LMA_OFFSET;
        . += 1M;
    } > kernel AT > kernel_lma

    /DISCARD/ : {
        *(.eh_frame*)
    }
}
