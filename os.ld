OUTPUT_FORMAT(binary);

ENTRY(os_main)

MEMORY{
    kernel (WX) : ORIGIN = 0xffffffff80000000, LENGTH = 0x20000000
}

SECTIONS
{

    .head 0xffffffff80000000 : {
        QUAD(os_main)
        QUAD(tail_addr - ADDR(.head))
    } > kernel

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

    tail_addr = .;
}
