OUTPUT_FORMAT(elf64-x86-64)

ENTRY(_start)

MEMORY
{
    boot (WX) : ORIGIN = 0x2000, LENGTH = 0x1000
}

SECTIONS
{
    . += SIZEOF_HEADERS;

    .multiboot ALIGN(64) : {
        *(.multiboot*)
    } > boot

    .text : {
        *(.text*)
    } > boot

    .data : {
        *(.data*)
    } > boot

    .rodata : {
        *(.rodata*)
    } > boot

    .bss : {
        *(.bss*)
    } > boot

    .eh_frame : {
        *(.eh_frame)
    } > boot
}
