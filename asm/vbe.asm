    VBE           EQU     0x9000
    BPP           EQU     0x0ff2          ; 色数に関する情報。何ビットカラーか？
    SCRNX         EQU     0x0ff4          ; 解像度のX
    SCRNY         EQU     0x0ff6          ; 解像度のY
    VRAM          EQU     0x0ff8          ; グラフィックバッファの開始番地
    VBEMODE       EQU     0x0ffc          ; VBE mode number. word size
    VBE_INFO_SIZE EQU     0x0200

    ; Get VBE Info
    MOV           AX,VBE
    MOV           ES,AX
    MOV           DI,0
    MOV           AX,0x4f00
    INT           0x10

    ; Loop initialization
    MOV           BYTE[BPP],8
    MOV           WORD[SCRNX],320
    MOV           WORD[SCRNY],200
    MOV           DI,VBE_INFO_SIZE

select_mode:
    VMODE_PTR     EQU 14
    ; Get VESA mode number
    MOV           SI,WORD[ES:VMODE_PTR]
    MOV           FS,WORD[ES:VMODE_PTR+2]
    MOV           CX,WORD[FS:SI]

    CMP           CX,0xffff
    JE            set_vbe

    ; Get VESA mode information.
    MOV           AX,0x4f01

    INT           0x10

    CMP           AX,0x004f
    JNE           next_mode

    ; Check if this graphics mode supports linear frame buffer support.
    MOV           AX,WORD[ES:DI]
    AND           AX,0x80
    JNE           next_mode

    ; Check if this is a packed pixel
    MOV           AX,WORD[ES:DI+27]
    CMP           AX,4
    JE            valid_mode

    ; Check if this is a direct color mode
    CMP           AX,6
    JE            valid_mode

    JMP           next_mode

valid_mode:
    ; Compare dimensions
    MOV           AX,WORD[ES:DI+18]
    CMP           AX,WORD[SCRNX]
    JB            next_mode

    MOV           AX,WORD[ES:DI+20]
    CMP           AX,WORD[SCRNY]
    JB            next_mode

    ; If bpp is not 24 bit or 32 bit, don't use this.
    CMP           BYTE[ES:DI+25],24
    JB            next_mode

    ; Set dimension and bits number
    MOV           AX,WORD[ES:DI+18]
    MOV           WORD[SCRNX],AX

    MOV           AX,WORD[ES:DI+20]
    MOV           WORD[SCRNY],AX

    MOV           AL,BYTE[ES:DI+25]
    MOV           BYTE[BPP],AL

    MOV           EAX,DWORD[ES:DI+40]
    MOV           DWORD[VRAM],EAX

    MOV           WORD[VBEMODE],CX

next_mode:
    ADD           WORD[ES:VMODE_PTR],2

    JMP           select_mode

set_vbe:
    MOV           AX,0x4f02
    MOV           BX,WORD[VBEMODE]
    OR            BX,0x4000
    INT           0x10


