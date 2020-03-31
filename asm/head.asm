    [BITS 64]
    ORG      0x500

    ; Disable all interrupts.
    MOV      AL,0xFF
    OUT      0x21,AL
    NOP                                  ; OUT命令を連続させるとうまくいかない機種があるらしいので
    OUT      0xA1,AL

    CLI                                  ; さらにCPUレベルでも割り込み禁止

    LGDT     [gdtr]
    PUSH     CODE_SEGMENT
    PUSH     change_code_segment
    RETFQ
change_code_segment:
    MOV      AX, DATA_SEGMENT
    MOV      ES, AX
    MOV      SS, AX
    MOV      DS, AX
    MOV      FS, AX
    MOV      GS, AX

    ;%include "paging_64.asm"

    MOV      RSP,0x00400000

    ; JMP 0xFFFFFFFF80000000 can't be executed.
    ; Jump to 64 bit immediate address is not supported.
    MOV      RDI,0x00200000
    JMP      RDI

gdtr:
    DW       gdt_end - gdt - 1
    DQ       gdt

gdt:
    TIMES    8  DB  0
    DATA_SEGMENT    EQU 0x08
    DW       0xFFFF, 0x0000, 0x9200, 0x00CF
    CODE_SEGMENT    EQU 0x10
    DW       0xFFFF, 0x0000, 0x9A00, 0x00AF
gdt_end:
