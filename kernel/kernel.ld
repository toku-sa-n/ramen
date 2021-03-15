OUTPUT_FORMAT(elf64-x86-64);

ENTRY(_start)

EXTERN(multiboot_header)

MEMORY{
    boot (RWX)       : ORIGIN = 0x1000,             LENGTH = 0x1000
    kernel (RWX)     : ORIGIN = 0xffffffff80000000, LENGTH = 0x8000000
    kernel_lma (RWX) : ORIGIN = 0x2000,             LENGTH = 0x8000000
}

SECTIONS
{
    . = 0x1000;

    .multiboot ALIGN(8) : {
        KEEP(*(.multiboot*))
    } > boot

    .boot : {
        *(.boot*)
    } > boot

    .text : {
        *(.text*)
    } > kernel AT > kernel_lma

    .data : {
        *(.data)
    } > kernel AT > kernel_lma

    .rodata : {
        *(.rodata*)
    } > kernel AT > kernel_lma

    .bss : {
        HEAP_START = .;
        . += 0x40000;
        HEAP_END = .;

        KERNEL_PML4 = ALIGN(4K);
        . += 4K;

        KERNEL_PDPT = ALIGN(4K);
        . += 4K;

        KERNEL_PD = ALIGN(4K);
        . += 4K;

        KERNEL_PT = ALIGN(4K);
        . += 4K;

        *(.bss)
    } > kernel AT > kernel_lma

    .eh_frame : {
        *(.eh_frame)
    } > kernel AT > kernel_lma
}
