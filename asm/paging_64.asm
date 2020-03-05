    ; Initialize PML4
    MOV             EAX, 0

    PML4            EQU 0x00100000
    MOV             EDI, PML4

    ; 1 PML4, 2 PDPT, 2PD and 2PT
    NUM_ALL_ENTRIES EQU 1024 * 7
    MOV             ECX, NUM_ALL_ENTRIES

    REP             STOSD
