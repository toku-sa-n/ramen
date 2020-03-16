OUTPUT_FORMAT(binary);

ENTRY(os_main)

SECTIONS
{
    .text 0xFFFFFFFF80000000 : {
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
