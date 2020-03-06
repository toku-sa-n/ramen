    ; Initialize PML4
    XOR                  EAX, EAX

    PML4                 EQU 0x00100000
    MOV                  EDI, PML4

    ; 1 PML4, 2 PDPT, 2PD and 2PT
    BYTES_PML4           EQU 0x1000
    BYTES_PDPT           EQU 0x1000
    BYTES_PD             EQU 0x1000
    BYTES_PT             EQU 0x1000
    NUM_ALL_ENTRIES      EQU BYTES_PML4 + 2 * BYTES_PDPT + 2 * BYTES_PD + 2 * BYTES_PT
    MOV                  ECX, NUM_ALL_ENTRIES

    REP                  STOSD

    ; Add a PML4 entry for below 1MB
    PDPT_BELOW_1MB       EQU PML4 + BYTES_PML4
    PAGE_EXISTS          EQU 1

    ; MOV [DWORD PML4] will cause an assemble error.
    ; MOV DWORD[PML4] won't cause any assemble errors, but it won't assign a value
    ; to ES:PML4.
    MOV                  DWORD[DWORD PML4], PDPT_BELOW_1MB | PAGE_EXISTS

    ; Add a PDPT entry for below 1MB
    PD_BELOW_1MB         EQU PDPT_BELOW_1MB + BYTES_PDPT
    MOV                  DWORD[DWORD PDPT_BELOW_1MB], PD_BELOW_1MB | PAGE_EXISTS

    ; Add a PD entry and PT entries for below 1MB
    XOR                  EAX, EAX

    PT_BELOW_1MB         EQU PD_BELOW_1MB + BYTES_PD
    MOV                  EBX, PT_BELOW_1MB
    MOV                  EDI, PD_BELOW_1MB
    MOV                  ECX, 1024 * 1024
    CALL                 map_entries

    ; Add a PML4 entry for kernel
    PML4_ENTRY_KERNEL    EQU PML4 + 0x1FF * SIZE_ENTRY
    PDPT_KERNEL          EQU PT_BELOW_1MB + BYTES_PT
    MOV                  DWORD[DWORD PML4_ENTRY_KERNEL], PDPT_KERNEL | PAGE_EXISTS

    ; Add a PDPT entry for kernel
    PDPT_ENTRY_KERNEL    EQU PDPT_KERNEL + 0x1FE * SIZE_ENTRY
    PD_KERNEL            EQU PDPT_KERNEL + BYTES_PDPT
    MOV                  DWORD[DWORD PDPT_ENTRY_KERNEL], PD_KERNEL | PAGE_EXISTS

    ; Add a PD entry and PT entries for kernel, IDT and stack.
    ; These three elements are located successively.
    ADDRESS_KERNEL       EQU 0x00501000
    MOV                  EAX, ADDRESS_KERNEL

    PT_KERNEL            EQU PD_KERNEL + BYTES_PD
    MOV                  EBX, PT_KERNEL
    MOV                  EDI, PD_KERNEL

    BYTES_KERNEL         EQU 512 * 1024
    BYTES_IDT            EQU 4 * 1024
    BYTES_STACK          EQU 8 * 1024
    MOV                  ECX, BYTES_KERNEL + BYTES_IDT + BYTES_STACK
    CALL                 map_entries

    ; Switching to 64-bit mode
    ; Disable paging
    MOV                  EAX, CR0
    AND                  EAX, 0x7FFFFFFF
    MOV                  CR0, EAX

    ; Enable PAE
    MOV                  EAX, CR4
    OR                   EAX, 0x00000020
    MOV                  CR4, EAX

    ; Set PML4 address
    MOV                  EAX, PML4
    MOV                  CR3, EAX

    ; Enable IA-32e mode
    MOV                  ECX, 0xC0000080
    RDMSR
    OR                   EAX, 0x00000100
    WRMSR

    ; Enable 4-level paging
    MOV                  EAX, CR0
    OR                   EAX, 0x80000000
    MOV                  CR0, EAX

    CODE_SEGMENT_64 EQU 0x18
    JMP                  CODE_SEGMENT_64:switch_to_64bit

    ; Functions

map_entries:
    ; Associate physical memories starting with EAX to page directory entries
    ; starting with EDI.
    ; Page table will be created successively from physical address EBX.
    ; EDX will be used as a temporary register.
    ;
    ; EAX: Starting address of physical memories.
    ; EBX: Starting address of page tables.
    ; EDI: Starting address of entries of a page directory.
    ; ECX: Number of bytes to map.
    PUSH                 EBP
    MOV                  EBP, ESP

    ; Number of entries = ECX / (bytes of a page table)
    ;                   = ECX >> 12
    SHR                  ECX, 12

loop_map_entries:
    ; The number of entries a 4-level page table contains is 512, not 1024.
    NUM_PAGE_ENTRIES     EQU 512
    CMP                  ECX, NUM_PAGE_ENTRIES
    JBE                  map_remainings

    MOV                  EDX, EBX
    OR                   EDX, PAGE_EXISTS
    MOV                  [EDI], EDX

    PUSH                 ECX,
    MOV                  ECX, NUM_PAGE_ENTRIES

    PUSH                 EDI
    MOV                  EDI, EBX
    CALL                 map_to_single_table

    POP                  EDI
    POP                  ECX

    SUB                  ECX, NUM_PAGE_ENTRIES

    SIZE_TABLE           EQU 0x1000
    ADD                  EBX, SIZE_TABLE

    ; The size of entry of 4-level paging is 8 bytes, not 4.
    SIZE_ENTRY           EQU 8
    ADD                  EDI, SIZE_ENTRY

    JMP                  loop_map_entries

map_remainings:
    MOV                  EDX, EBX
    OR                   EDX, PAGE_EXISTS
    MOV                  [EDI], EDX

    MOV                  EDI, EBX
    CALL                 map_to_single_table

    MOV                  ESP, EBP
    POP                  EBP
    RET

map_to_single_table:
    ; Map ECX entries to a page table.
    ; (EDI - (page directory base address)) / 4 + ECX must be less than or equal to 1024.
    ;
    ; EAX: Starting address of physical memories.
    ; ECX: Number of entries to map.
    ; EDI: Starting address of entries of a page table.
    ; EDX will be used as a temporary register.
    PUSH                 EBP
    MOV                  EBP, ESP

loop_map_to_single_table:
    CMP                  ECX, 0
    JBE                  end_map_to_single_table

    MOV                  EDX, EAX
    OR                   EDX, PAGE_EXISTS
    MOV                  [EDI], EDX

    ADD                  EAX, 0x1000
    ADD                  EDI, SIZE_ENTRY
    DEC                  ECX

    JMP                  loop_map_to_single_table

end_map_to_single_table:
    MOV                  ESP, EBP
    POP                  EBP
    RET

switch_to_64bit:
    [BITS 64]
