    .section .multiboot
    .intel_syntax noprefix

    .extern _start

    .align 64

start_multiboot_header:

    .set MAGIC_NUMBER, 0xe85250d6
    .set ARCHITECTURE_I386, 0
    .set HEADER_SIZE, end_multiboot_header - start_multiboot_header
    .set CHECKSUM, -(MAGIC_NUMBER + ARCHITECTURE_I386 + HEADER_SIZE)

    .long MAGIC_NUMBER
    .long ARCHITECTURE_I386
    .long HEADER_SIZE
    .long CHECKSUM

    .set TERMINATION, 0
    .set TERMINATION_SIZE, 8

    .word 9
    .word 0
    .long 12
    .long 0x80000000

    .word TERMINATION
    .word TERMINATION_SIZE

end_multiboot_header:
