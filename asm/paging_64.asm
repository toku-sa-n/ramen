    ; Initialize PML4
    MOV             EAX, 0

    PML4            EQU 0x00100000
    MOV             EDI, PML4

    ; 1 PML4, 2 PDPT, 2PD and 2PT
    NUM_ALL_ENTRIES EQU 1024 * 7
    MOV             ECX, NUM_ALL_ENTRIES

    REP             STOSD

    ; Add a PML4 entry for below 1MB
    PDPT_BELOW_1MB  EQU PML4 + 0x1000
    PAGE_EXISTS     EQU 1

    ; MOV [DWORD PML4] will cause an assemble error.
    ; MOV DWORD[PML4] won't cause any assemble errors, but it won't assign a value
    ; to ES:PML4.
    MOV             DWORD[DWORD PML4], PDPT_BELOW_1MB | PAGE_EXISTS
