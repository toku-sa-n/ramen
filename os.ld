OUTPUT_FORMAT(binary);
OUTPUT_ARCH(i386);

ENTRY(os_main)

SECTIONS
{
    .text 0xFFFFFFFF80000000 : { *(.text*) }

    .data : {
        *(.data)
        *(.rodata*)
        *(.bss)
    }

    /DISCARD/ : { *(.eh_frame) }
}
