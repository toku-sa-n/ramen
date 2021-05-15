OUTPUT_FORMAT(elf64-x86-64);

ENTRY(main);

SECTIONS
{
    /* FIXME: This is a workaround to avoid memory overlap of the stack memory and the binary image. Remove this line if the problem is solved. */
    . = 0x800000;

    .text : {
        *(.text*)
    }

    .data : {
        *(.data)
    }

    .rodata : {
        *(.rodata*)
    }

    .bss : {
        HEAP_START = .;
        . += 0x10000;
        HEAP_END = .;
        *(.bss)
    }

    .eh_frame : {
        *(.eh_frame)
    }
}
