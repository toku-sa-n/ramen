    ; Initialize page directory and page tables with 0

    MOV             EAX, 0

    DIR             EQU 0x00100000
    MOV             EDX, DIR

    NUM_ALL_ENTRIES EQU 1024 + 1024 * 1024
    MOV             ECX, NUM_ALL_ENTRIES

    REP             STOSD

    ; Map kernel
    ; Add an entry for kernel to page directory

    SIZE_TABLE      EQU 0x1000
    TABLE_KERNEL    EQU DIR + SIZE_TABLE
    SIZE_ENTRY      EQU 4
    DIR_ENTRY_KERNEL    EQU DIR + 0x300 * SIZE_ENTRY
    PAGE_EXISTS     EQU 1
    MOV             DWORD[DWORD DIR_ENTRY_KERNEL], TABLE_KERNEL | PAGE_EXISTS
