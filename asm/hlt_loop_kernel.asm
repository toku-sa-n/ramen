    [BITS 64]
    EXTERN os_main

os_main:
    HLT
    JMP os_main
