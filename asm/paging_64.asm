    ; |Address     |Conversion table                    |
    ; |------------|------------------------------------|
    ; |0x00100000  |PML4                                |
    ; |0x00101000  |PDPT for below 1MB                  |
    ; |0x00102000  |PD   for below 1MB                  |
    ; |0x00103000  |PT   for below 1MB                  |
    ; |0x00104000  |PDPT for kernel, IDT, stack and VRAM|
    ; |0x00105000  |PD   for kernel, IDT, stack and VRAM|
    ; |0x00106000  |PT   for kernel, IDT and stack      |
    ; |0x00107000 ~|PT   for VRAM.                      |

    [BITS 64]
    XOR                  EAX, EAX
    MOV                  RCX, 0x00100000 / 8
    MOV                  RDI, 0x00100000
    REP                  STOSQ

    PML4                 EQU 0x00100000
    BYTES_PML4           EQU 0x1000
    BYTES_PDPT           EQU 0x1000
    BYTES_PD             EQU 0x1000
    BYTES_PT             EQU 0x1000

    ; Add a PML4 entry for below 1MB
    PDPT_BELOW_1MB       EQU PML4 + BYTES_PML4
    PAGE_EXISTS          EQU 1

    ; MOV [DWORD PML4] will cause an assemble error.
    ; MOV DWORD[PML4] won't cause any assemble errors, but it won't assign a value
    ; to ES:PML4.
    MOV                  RAX, PDPT_BELOW_1MB | PAGE_EXISTS
    MOV                  QWORD[DWORD PML4], RAX

    ; Add a PDPT entry for below 1MB
    PD_BELOW_1MB         EQU PDPT_BELOW_1MB + BYTES_PDPT
    MOV                  RAX, PD_BELOW_1MB | PAGE_EXISTS
    MOV                  QWORD[DWORD PDPT_BELOW_1MB], RAX

    ; Add a PD entry and PT entries for below 1MB
    XOR                  EAX, EAX

    PT_BELOW_1MB         EQU PD_BELOW_1MB + BYTES_PD
    MOV                  RBX, PT_BELOW_1MB
    MOV                  RDI, PD_BELOW_1MB
    MOV                  RCX, 1024 * 1024
    CALL                 map_entries

    ; Add a PML4 entry for kernel
    PML4_ENTRY_KERNEL    EQU PML4 + 0x1FF * SIZE_ENTRY
    PDPT_KERNEL          EQU PT_BELOW_1MB + BYTES_PT
    MOV                  RAX, PDPT_KERNEL | PAGE_EXISTS
    MOV                  QWORD[DWORD PML4_ENTRY_KERNEL], RAX

    ; Add a PDPT entry for kernel
    PDPT_ENTRY_KERNEL    EQU PDPT_KERNEL + 0x1FE * SIZE_ENTRY
    PD_KERNEL            EQU PDPT_KERNEL + BYTES_PDPT
    MOV                  RAX, PD_KERNEL | PAGE_EXISTS
    MOV                  QWORD[DWORD PDPT_ENTRY_KERNEL], RAX

    ; Add a PD entry and PT entries for kernel, IDT and stack.
    ; These three elements are located successively.
    ADDRESS_KERNEL       EQU 0x00200000
    MOV                  EAX, ADDRESS_KERNEL

    PT_KERNEL            EQU PD_KERNEL + BYTES_PD
    MOV                  EBX, PT_KERNEL
    MOV                  EDI, PD_KERNEL

    BYTES_KERNEL         EQU 512 * 1024
    BYTES_IDT            EQU 4 * 1024
    BYTES_STACK          EQU 128 * 1024
    MOV                  ECX, BYTES_KERNEL + BYTES_IDT + BYTES_STACK
    CALL                 map_entries

    ; Calculate bytes used by VRAM.
    XOR                  EAX, EAX

    VRAM_BPP             EQU 0x0FF2
    MOV                  AL, [VRAM_BPP]
    SHR                  AX, 3
    XOR                  EBX, EBX

    VRAM_X               EQU 0x0FF4
    MOV                  BX, [VRAM_X]

    ; The result of MUL instruction will be stored in EDX:EAX.
    ; However, the resolution should be less than 2**32 = 4294967296.
    ; The width of height must be less than 2**16 = 65536.
    ; This is why this program won't touch EDX.
    MUL                  EBX

    VRAM_Y               EQU 0x0FF6
    MOV                  BX, [VRAM_Y]
    MUL                  EBX
    MOV                  ECX, EAX

    ; Add a PD entry and PT entries for VRAM.
    VRAM_PTR             EQU 0x0FF8
    MOV                  EAX, [VRAM_PTR]

    PD_ENTRY_VRAM        EQU PD_KERNEL + SIZE_ENTRY
    MOV                  EDI, PD_ENTRY_VRAM

    PT_VRAM              EQU PT_KERNEL + BYTES_PT
    MOV                  EBX, PT_VRAM
    CALL                 map_entries

    ; Set PML4 address
    MOV                  RAX, PML4
    MOV                  CR3, RAX

    ; Clear write protection
    ; See: https://forum.osdev.org/viewtopic.php?f=1&t=36615
    MOV                  RAX, CR0
    AND                  EAX, 0xFFFEFFFF
    MOV                  CR0, RAX

    ; Replace pointer to the physical address of VRAM to the virtual one.
    MOV                  RAX, 0xFFFFFFFF80200000
    MOV                  QWORD[VRAM_PTR], RAX
    JMP                  finish_paging_setting

    ; Functions

map_entries:
    ; Associate physical memories starting with RAX to page directory entries
    ; starting with RDI.
    ; Page table will be created successively from physical address RBX.
    ; RDX will be used as a temporary register.
    ;
    ; RAX: Starting address of physical memories.
    ; RBX: Starting address of page tables.
    ; RDI: Starting address of entries of a page directory.
    ; ECX: Number of bytes to map.
    PUSH                 RBP
    MOV                  RBP, RSP

    ; Number of entries = ECX / (bytes of a page table)
    ;                   = ECX >> 12
    SHR                  ECX, 12

loop_map_entries:
    ; The number of entries a 4-level page table contains is 512, not 1024.
    NUM_PAGE_ENTRIES     EQU 512
    CMP                  ECX, NUM_PAGE_ENTRIES
    JBE                  map_remainings

    MOV                  RDX, RBX
    OR                   RDX, PAGE_EXISTS
    MOV                  [RDI], EDX

    PUSH                 RCX,
    MOV                  ECX, NUM_PAGE_ENTRIES

    PUSH                 RDI
    MOV                  RDI, RBX
    CALL                 map_to_single_table

    POP                  RDI
    POP                  RCX

    SUB                  ECX, NUM_PAGE_ENTRIES

    SIZE_TABLE           EQU 0x1000
    ADD                  RBX, SIZE_TABLE

    ; The size of entry of 4-level paging is 8 bytes, not 4.
    SIZE_ENTRY           EQU 8
    ADD                  RDI, SIZE_ENTRY

    JMP                  loop_map_entries

map_remainings:
    MOV                  RDX, RBX
    OR                   RDX, PAGE_EXISTS
    MOV                  [RDI], RDX

    MOV                  RDI, RBX
    CALL                 map_to_single_table

    MOV                  RSP, RBP
    POP                  RBP
    RET

map_to_single_table:
    ; Map ECX entries to a page table.
    ; (EDI - (page directory base address)) / 4 + ECX must be less than or equal to 1024.
    ;
    ; RAX: Starting address of physical memories.
    ; ECX: Number of entries to map.
    ; EDI: Starting address of entries of a page table.
    ; RDX will be used as a temporary register.
    PUSH                 RBP
    MOV                  RBP, RSP

loop_map_to_single_table:
    CMP                  ECX, 0
    JBE                  end_map_to_single_table

    MOV                  RDX, RAX
    OR                   RDX, PAGE_EXISTS
    MOV                  [EDI], RDX

    ADD                  RAX, 0x1000
    ADD                  RDI, SIZE_ENTRY
    DEC                  ECX

    JMP                  loop_map_to_single_table

end_map_to_single_table:
    MOV                  RSP, RBP
    POP                  RBP
    RET

finish_paging_setting:
