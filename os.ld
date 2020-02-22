OUTPUT_FORMAT(binary);
OUTPUT_ARCH(i386);

ENTRY(os_main)

SECTIONS
{
    .text : { *(.text*) }

    .data ADDR(.text) + SIZEOF(.text) + 0x00501000 : AT ( ADDR(.text) + SIZEOF(.text) ) {
        *(.data)
        *(.rodata*)
        *(.bss)
    }

    /DISCARD/ : { *(.eh_frame) }
}
