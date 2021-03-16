OUTPUT_FORMAT(elf64-x86-64);

ENTRY(_start)

EXTERN(multiboot_header)

MEMORY{
    boot (RWX)       : ORIGIN = 0,                  LENGTH = 0x1000
    kernel (RWX)     : ORIGIN = 0xffffffff80000000, LENGTH = 0x8000000
    kernel_lma (RWX) : ORIGIN = 0x1000,             LENGTH = 0x8000000
}

KERNEL_LMA_OFFSET = 0xffffffff80000000 - 0x1000;

SECTIONS
{
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

    .data : {
        *(.data*)
    } > kernel AT > kernel_lma

    .rodata : {
        *(.rodata*)
    } > kernel AT > kernel_lma

    .bss : {
        HEAP_START = .;
        . += 0x40000;
        HEAP_END = .;

        . = ALIGN(4K);
        KERNEL_PML4_LMA = . - KERNEL_LMA_OFFSET;
        . += 4K;

        KERNEL_PDPT_LMA = . - KERNEL_LMA_OFFSET;
        . += 4K;

        /* TODO: Remove this. */
        LONG(0)

        . += 8K;
        . = ALIGN(16);
        KERNEL_STACK = .;
        KERNEL_STACK_LMA = . - KERNEL_LMA_OFFSET;

        MULTIBOOT2_INFORMATION_LMA = . - KERNEL_LMA_OFFSET;
        . += 1M;

        *(.bss*)
    } > kernel AT > kernel_lma

    .eh_frame : {
        *(.eh_frame*)
    } > kernel AT > kernel_lma
}
