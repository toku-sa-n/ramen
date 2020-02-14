OUTPUT_FORMAT(binary);
OUTPUT_ARCH(i386);

ENTRY(os_main)

SECTIONS
{
    .head 0 : {
        LONG(64 * 1024)
        LONG(0x69726148)
        LONG(0)
        LONG(ADDR(.data))
        LONG(SIZEOF(.data))
        LONG(LOADADDR(.data))
        LONG(0xE9000000)
        LONG(os_main - 0x20)
        LONG(0)
    }

    .text : { *(.text*) }

    .data 0x310000 : AT ( ADDR(.text) + SIZEOF(.text) ) {
        *(.data)
        *(.rodata*)
        *(.bss)
    }

    /DISCARD/ : { *(.eh_frame) }
}
