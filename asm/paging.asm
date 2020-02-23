    ; Initialize page directory and page tables with 0

    MOV             EAX, 0

    DIR             EQU 0x00100000
    MOV             EDX, DIR

    NUM_ALL_ENTRIES EQU 1024 + 1024 * 1024
    MOV             ECX, NUM_ALL_ENTRIES

    REP             STOSD
