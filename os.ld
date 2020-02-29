OUTPUT_FORMAT(binary);
OUTPUT_ARCH(i386);

ENTRY(os_main)

SECTIONS
{
    .text 0xC0000000 : { *(.text*) }

    .data : {
        *(.data)
        *(.rodata*)
        *(.bss)
    }

    /DISCARD/ : { *(.eh_frame) }
}
