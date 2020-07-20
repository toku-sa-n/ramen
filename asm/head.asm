    [BITS 64]
    ORG      0x8000

    ; Disable all interrupts.
    MOV      AL,0xFF
    OUT      0x21,AL
    NOP                                  ; OUT命令を連続させるとうまくいかない機種があるらしいので
    OUT      0xA1,AL

    CLI                                  ; さらにCPUレベルでも割り込み禁止

    %include "paging_64.asm"

    MOV      RSP,0xFFFFFFFF800a1000

    ; JMP 0xFFFFFFFF80000000 can't be executed.
    ; Jump to 64 bit immediate address is not supported.
    MOV      RDI,0xFFFFFFFF80000000
    JMP      RDI
