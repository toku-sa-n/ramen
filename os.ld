OUTPUT_FORMAT(binary);

ENTRY(os_main)

MEMORY{
    kernel (WX) : ORIGIN = 0xffffffff80000000, LENGTH = 0x20000000
}

SECTIONS
{
    .head 0xffffffff80000000 : {
        QUAD(os_main)
        QUAD(SIZEOF(.head) + SIZEOF(.text) + SIZEOF(.data))
    } > kernel

    .text : {
        *(.text*)
    } > kernel

    .data : {
        *(.data)
        *(.rodata*)
        *(.bss)
    } > kernel

    /DISCARD/ : { *(.eh_frame) }
}
