OUTPUT_FORMAT(elf64-x86-64);

ENTRY(_start)

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
        *(.bss)
    } > kernel AT > kernel_lma

    .eh_frame : {
        *(.eh_frame)
    } > kernel AT > kernel_lma
}
