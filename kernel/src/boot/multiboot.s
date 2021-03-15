// SPDX-License-Identifier: GPL-3.0-or-later

    .intel_syntax noprefix
    .section .multiboot, "a"
    .global multiboot_header
    .code32
    .align 64

multiboot_header:

    .set MAGIC_NUMBER, 0xe85250d6
    .set ARCHITECTURE_I386, 0
    .set HEADER_SIZE, end_multiboot_header - multiboot_header
    .set CHECKSUM, -(MAGIC_NUMBER + ARCHITECTURE_I386 + HEADER_SIZE)

    .long MAGIC_NUMBER
    .long ARCHITECTURE_I386
    .long HEADER_SIZE
    .long CHECKSUM

    .set TERMINATION, 0
    .set TERMINATION_SIZE, 8

    .word TERMINATION
    .word TERMINATION_SIZE

end_multiboot_header:
