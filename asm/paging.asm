    ; Initialize page directory and page tables with 0

    MOV                  EAX, 0

    DIR                  EQU 0x00100000
    MOV                  EDX, DIR

    NUM_ALL_ENTRIES      EQU 1024 + 1024 * 1024
    MOV                  ECX, NUM_ALL_ENTRIES

    REP                  STOSD

    ; Map kernel
    ; Add an entry for kernel to page directory

    SIZE_TABLE           EQU 0x1000
    TABLE_KERNEL         EQU DIR + SIZE_TABLE
    SIZE_ENTRY           EQU 4
    DIR_ENTRY_KERNEL     EQU DIR + 0x300 * SIZE_ENTRY
    PAGE_EXISTS          EQU 1

    ; Map kernel to page table.
    ADDRESS_KERNEL      EQU  0x00501000
    MOV                 EAX, ADDRESS_KERNEL

    SIZE_KERNEL         EQU  512 * 1024
    MOV                 ECX, SIZE_KERNEL
    MOV                 EBX, TABLE_KERNEL
    MOV                 EDI, DIR_ENTRY_KERNEL
    CALL                map_entries

    ; Add a page table entry for IDT
    ADDRESS_IDT         EQU 0x00581000
    TABLE_ENTRY_IDT     EQU TABLE_KERNEL + SIZE_KERNEL / 1024
    MOV                 DWORD[DWORD TABLE_ENTRY_IDT], ADDRESS_IDT | PAGE_EXISTS

    ; Add a page table entry for stack.
    ADDRESS_STACK       EQU 0x00582000
    TABLE_ENTRY_STACK   EQU TABLE_ENTRY_IDT + SIZE_ENTRY
    MOV                 DWORD[DWORD TABLE_ENTRY_STACK], ADDRESS_STACK | PAGE_EXISTS

    ; Map below 1MB to page table.
    TABLE_BELOW_1MB     EQU DIR + SIZE_TABLE * 2
    MOV                 EAX, 0
    MOV                 ECX, 1024 * 1024
    MOV                 EBX, TABLE_BELOW_1MB
    MOV                 EDI, DIR
    CALL                map_entries

    ; Calculate the number of bytes of VRAM.
    VRAM_BPP            EQU 0x0ff2
    XOR                 EAX, EAX
    MOV                 AL, BYTE[VRAM_BPP]
    SHR                 EAX, 3

    VRAM_X              EQU 0x0ff4
    XOR                 EBX, EBX
    MOV                 BX, WORD[VRAM_X]
    MUL                 EBX

    VRAM_Y              EQU 0x0ff6
    XOR                 EBX, EBX
    MOV                 BX, WORD[VRAM_Y]
    MUL                 EBX

    MOV                 ECX, EAX

    VRAM_PTR            EQU 0x0ff8
    MOV                 EAX, [VRAM_PTR]

    TABLE_VRAM          EQU TABLE_BELOW_1MB + SIZE_TABLE
    MOV                 EBX, TABLE_VRAM
    MOV                 EDI, DIR_ENTRY_KERNEL + SIZE_ENTRY
    CALL                map_entries

    MOV                 EAX, DIR
    MOV                 CR3, EAX

    MOV                 EAX, CR0
    OR                  EAX, 0x80000000
    MOV                 CR0, EAX

    JMP                 end_page_settings

    ; Function

map_entries:
    ; EAX: Starting address of physical memories.
    ; EBX: Starting address of page table.
    ; EDI: Starting address of entries of page directory.
    ; ECX: Number of bytes to map.
    PUSH                EBP
    MOV                 EBP, ESP

    ; Number of needed entries = ECX / 4096
    ;                          = ECX >> 12
    SHR                 ECX, 12

loop_map_entries:
    NUM_PAGE_ENTRIES    EQU  1024
    CMP                 ECX, NUM_PAGE_ENTRIES
    JBE                 end_map_entries

    OR                  EBX,   PAGE_EXISTS
    MOV                 [EDI], EBX
    AND                 EBX, 0xFFFFF000

    PUSH                ECX
    MOV                 ECX, NUM_PAGE_ENTRIES

    PUSH                EDI
    MOV                 EDI, EBX
    CALL                map_to_single_table

    POP                 EDI
    POP                 ECX

    SUB                 ECX, NUM_PAGE_ENTRIES
    ADD                 EBX, SIZE_TABLE
    ADD                 EDI, SIZE_ENTRY

    JMP                 loop_map_entries

end_map_entries:
    CMP                 ECX, 0
    JBE                 return_from_map_entries

    OR                  EBX, PAGE_EXISTS
    MOV                 [EDI], EBX
    AND                 EBX, 0xFFFFF000

    MOV                 EDI, EBX
    CALL                map_to_single_table

return_from_map_entries:
    MOV                 ESP, EBP
    POP                 EBP
    RET

map_to_single_table:
    ; EAX: Starting address of physical memories.
    ; ECX: Number of entries to map.
    ; EDI: Starting address of page table.
    PUSH                EBP
    MOV                 EBP, ESP

loop_map_to_single_table:
    CMP                 ECX, 0
    JBE                 end_map_to_single_table

    OR                  EAX, PAGE_EXISTS
    MOV                 [EDI], EAX
    ADD                 EAX, 0x1000
    ADD                 EDI, SIZE_ENTRY
    DEC                 ECX

    JMP                 loop_map_to_single_table

end_map_to_single_table:
    MOV                 ESP, EBP
    POP                 EBP
    RET

end_page_settings:
