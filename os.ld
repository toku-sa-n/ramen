OUTPUT_FORMAT(binary);
OUTPUT_ARCH(i386);

ENTRY(os_main)

SECTIONS
{
    .head 0 : {
        LONG(0xE9000000)
        LONG(os_main - SIZEOF(.head))
    }

    .text : { *(.text*) }

    .data 0xFFFFFFFF80000000 + ADDR(.text) + SIZEOF(.text) : AT ( ADDR(.text) + SIZEOF(.text) ) {
        *(.data)
        *(.rodata*)
        *(.bss)
    }

    /DISCARD/ : { *(.eh_frame) }
}
