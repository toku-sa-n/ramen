    ; Initialize page directory and page tables with 0

    MOV                  EAX, 0

    DIR                  EQU 0x00100000
    MOV                  EDI, DIR

    NUM_ALL_ENTRIES      EQU 1024 + 1024 * 1024
    MOV                  ECX, NUM_ALL_ENTRIES

    REP                  STOSD

    ; Map kernel
    ADDRESS_KERNEL      EQU  0x00501000
    MOV                 EAX, ADDRESS_KERNEL

    SIZE_KERNEL         EQU  512 * 1024
    MOV                 ECX, SIZE_KERNEL

    SIZE_TABLE          EQU 0x1000
    TABLE_KERNEL        EQU DIR + SIZE_TABLE
    MOV                 EBX, TABLE_KERNEL

    SIZE_ENTRY          EQU 4

    ; Don't forget to multiply the index of entry by size of entry.
    DIR_ENTRY_KERNEL    EQU DIR + 0x300 * SIZE_ENTRY
    MOV                 EDI, DIR_ENTRY_KERNEL
    CALL                map_entries

    ; Add a page table entry for IDT
    ADDRESS_IDT         EQU 0x00581000
    TABLE_ENTRY_IDT     EQU TABLE_KERNEL + SIZE_KERNEL / 1024
    PAGE_EXISTS         EQU 1

    ; If the least bit of entry is 1, page the entry points to exists.
    ; See Intel Architecture Software Developer Manuals Volume 3A, 4-11 Figure 4-4, Table 4-5, Table 4-6.
    ;
    ; Don't forget to add the prefix DWORD before TABLE_ENTRY_IDT.
    ; [DWORD TABLE_ENTRY_IDT] or DWORD[TABLE_ENTRY_IDT] won't work!
    ; nasm will cause an error to the former, but won't warn to the latter even though
    ; the code assigns nothing!
    MOV                 DWORD[DWORD TABLE_ENTRY_IDT], ADDRESS_IDT | PAGE_EXISTS

    ; Add a page table entry for stack.
    ; If you change the size of stack, don't forget to change the initial value of ESP
    ; which is defined in head.asm.
    ADDRESS_STACK       EQU 0x00582000
    MOV                 EAX, ADDRESS_STACK
    MOV                 ECX, 32

    ; Page entries for stack will be created in the same page table as kernel and IDT.
    TABLE_ENTRY_STACK   EQU TABLE_ENTRY_IDT + SIZE_ENTRY
    MOV                 EDI, TABLE_ENTRY_STACK
    CALL                map_to_single_table

    ; Map below 1MB to page table.
    ; This process is needed because IPL copies the contents of floppy to below 1MB area.
    ; Without this process, EIP will point the invalid memory address immediately after enabling paging.
    TABLE_BELOW_1MB     EQU DIR + SIZE_TABLE * 2
    MOV                 EAX, 0
    MOV                 ECX, 1024 * 1024
    MOV                 EBX, TABLE_BELOW_1MB
    MOV                 EDI, DIR
    CALL                map_entries

    ; Calculate the number of bytes of VRAM.
    VRAM_BPP            EQU 0x0FF2

    ; This is equivalent to MOV EAX, 0.
    ; See https://stackoverflow.com/questions/33666617/what-is-the-best-way-to-set-a-register-to-zero-in-x86-assembly-xor-mov-or-and
    XOR                 EAX, EAX

    ; MOV EAX, BYTE[VRAM_BPP] isn't possible. Instead, initialize EAX with 0 and MOV AL, BYTE[VRAM_BPP].
    MOV                 AL, BYTE[VRAM_BPP]

    ; The unit of [VRAM_BPP] is pixel.
    SHR                 EAX, 3

    VRAM_X              EQU 0x0FF4
    XOR                 EBX, EBX
    MOV                 BX, WORD[VRAM_X]

    ; MUL EBX will assign the result to EDX:EAX. However, the number of bytes of VRAM should be less than 2**32.
    ; 2**16 = 65536 is too large for width or height of a screen.
    ; This is why the codes don't touch EDX.
    MUL                 EBX

    VRAM_Y              EQU 0x0FF6
    XOR                 EBX, EBX
    MOV                 BX, WORD[VRAM_Y]
    MUL                 EBX

    MOV                 ECX, EAX

    VRAM_PTR            EQU 0x0FF8
    MOV                 EAX, [VRAM_PTR]

    TABLE_VRAM          EQU TABLE_BELOW_1MB + SIZE_TABLE
    MOV                 EBX, TABLE_VRAM
    MOV                 EDI, DIR_ENTRY_KERNEL + SIZE_ENTRY
    CALL                map_entries

    ; Replace physical pointer to VRAM to virtual one.
    ; Currently there's no need to keep physical one.
    MOV                 DWORD[VRAM_PTR], 0xC0400000

    MOV                 EAX, DIR
    MOV                 CR3, EAX

    MOV                 EAX, CR0
    OR                  EAX, 0x80000000
    MOV                 CR0, EAX

    JMP                 end_page_settings

    ; Function

map_entries:
    ; Map for ECX bytes from physical address EAX.
    ; The address of entries of page directory begins with EDI.
    ; Page table will be created successively from physical address EBX.
    ;
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

    ; Assign 0 to all flags not to cause any troubles.
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
    ; Map ECX entries to a single page table.
    ; (EDI - (page directory base address)) / 4 + ECX must be less than or equal to 1024.
    ;
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
