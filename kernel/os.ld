OUTPUT_FORMAT(elf64-x86-64);

ENTRY(os_main)

MEMORY{
    kernel (WX) : ORIGIN = 0xffffffff80000000, LENGTH = 0x10000000
}

SECTIONS
{

    . += 0xffffffff80000000 + SIZEOF_HEADERS;

    .text BLOCK(4K) : ALIGN(4K) {
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
