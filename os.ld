OUTPUT_FORMAT(elf64-x86-64);

ENTRY(os_main)

MEMORY{
    kernel (WX) : ORIGIN = 0xffffffff80000000, LENGTH = 0x20000000
}

SECTIONS
{

    . = 0xffffffff80000000;

    .text : {
        *(.text*)
    } > kernel

    .data : {
        *(.data)
    } > kernel

    .rodata : {
        *(.rodata*)
    } > kernel

    .bss : {
        *(.bss)
    } > kernel

    .eh_frame : {
        *(.eh_frame)
    } > kernel
}
