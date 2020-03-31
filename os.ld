OUTPUT_FORMAT(binary);

ENTRY(os_main)

SECTIONS
{
    .text 0x00200000 : {
        *(.text.os_main)
        *(.text*)
    }

    .data : {
        *(.data)
        *(.rodata*)
        *(.bss)
    }

    /DISCARD/ : { *(.eh_frame) }
}
