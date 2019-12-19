; haribote-os boot asm
; TAB=4

BOTPAK  EQU     0x00280000      ; bootpackのロード先
DSKCAC  EQU     0x00100000      ; ディスクキャッシュの場所
DSKCAC0 EQU     0x00008000      ; ディスクキャッシュの場所（リアルモード）

; BOOT_INFO関係
CYLS    EQU     0x0ff0          ; ブートセクタが設定する
LEDS    EQU     0x0ff1

VMODE   EQU     0x0ff2          ; 色数に関する情報。何ビットカラーか？
SCRNX   EQU     0x0ff4          ; 解像度のX
SCRNY   EQU     0x0ff6          ; 解像度のY
VRAM    EQU     0x0ff8          ; グラフィックバッファの開始番地
VBEMODE EQU     0x0ffc          ; VBE mode number. word size
VBE_INFO_SIZE EQU 0x0200

VBE     EQU     0x9000

        ORG     0xc200          ; このプログラムがどこに読み込まれるのか
; If VBE doesn't exist, the resolution will be 320x200
        MOV     AX,VBE
        MOV     ES,AX
        MOV     DI,0
        MOV     AX,0x4f00
        INT     0x10
        CMP     AX,0x004f
        JNE     screen_320

; If the version of VBE is less than 2.0, set the resolution as 320x200
        MOV     AX,WORD[ES:DI+4]
        CMP     AX,0x0200
        JB      screen_320

; Loop initialization
        MOV     BYTE[VMODE],8
        MOV     WORD[SCRNX],320
        MOV     WORD[SCRNY],200
        MOV     DI,VBE_INFO_SIZE
select_mode:

VMODE_PTR EQU 14
; Get VESA mode number
        MOV     SI,WORD[ES:VMODE_PTR]
        MOV     FS,WORD[ES:VMODE_PTR+2]
        MOV     CX,WORD[FS:SI]

        CMP     CX,0xffff
        JE      finish_select_mode

; Get VESA mode information.
        MOV     AX,0x4f01

        INT     0x10

        CMP     AX,0x004f
        JNE     next_mode

; Check if this graphics mode supports linear frame buffer support.
        MOV     AX,WORD[ES:DI]
        AND     AX,0x80
        CMP     AX,0x80
        JNE     next_mode

; Check if this is a packed pixel
        MOV     AX,WORD[ES:DI+27]
        CMP     AX,4
        JE      valid_mode

; Check if this is a direct color mode
        CMP     AX,6
        JE      valid_mode

        JMP     next_mode

valid_mode:
; Compare dimensions
        MOV     AX,WORD[ES:DI+18]
        CMP     AX,WORD[SCRNX]
        JB      next_mode

        MOV     AX,WORD[ES:DI+20]
        CMP     AX,WORD[SCRNY]
        JB      next_mode

        MOV     AX,WORD[ES:DI+25]
        CMP     AX,WORD[VMODE]
        JB      next_mode

; Set dimension and bits number
        MOV     AX,WORD[ES:DI+18]
        MOV     WORD[SCRNX],AX

        MOV     AX,WORD[ES:DI+20]
        MOV     WORD[SCRNY],AX

        MOV     AX,WORD[ES:DI+25]
        MOV     WORD[VMODE],AX

        MOV     AX,WORD[ES:DI+40]
        MOV     WORD[VRAM],AX
        MOV     AX,WORD[ES:DI+40+2]
        MOV     WORD[VRAM+2],AX

        MOV     WORD[VBEMODE],CX

next_mode:
        MOV     AX,WORD[ES:VMODE_PTR+2]
        ADD     AX,2
        MOV     WORD[ES:VMODE_PTR+2],AX

        JMP     select_mode

finish_select_mode:
        CMP     WORD[SCRNX],320
        JNE     set_vbe_mode

        CMP     WORD[SCRNY],200
        JNE     set_vbe_mode

        CMP     WORD[VMODE],8
        JNE     set_vbe_mode

        JMP     screen_320

set_vbe_mode:
        MOV     AX,0x4f02
        MOV     BX,WORD[VBEMODE]
        OR      BX,0x4000
        INT     0x10

        CMP     AX,0x004f
        JE      keystatus

screen_320:
        MOV     AL,0x13
        MOV     AH,0x00
        INT     0x10
        MOV     BYTE [VMODE],8
        MOV     WORD [SCRNX],320
        MOV     WORD [SCRNY],200

; DO NOT FOLLOW THE INSTRUCTIONS WRITTEN IN BOOK!
; SEE https://qiita.com/tatsumack/items/491e47c1a7f0d48fc762
        MOV     DWORD [VRAM],0xfd000000

keystatus:
; キーボードのLED状態をBIOSに教えてもらう

        MOV     AH,0x02
        INT     0x16            ; keyboard BIOS
        MOV     [LEDS],AL

; PICが一切の割り込みを受け付けないようにする
;   AT互換機の仕様では、PICの初期化をするなら、
;   こいつをCLI前にやっておかないと、たまにハングアップする
;   PICの初期化はあとでやる

        MOV     AL,0xff
        OUT     0x21,AL
        NOP                     ; OUT命令を連続させるとうまくいかない機種があるらしいので
        OUT     0xa1,AL

        CLI                     ; さらにCPUレベルでも割り込み禁止

; CPUから1MB以上のメモリにアクセスできるように、A20GATEを設定

        CALL    waitkbdout
        MOV     AL,0xd1
        OUT     0x64,AL
        CALL    waitkbdout
        MOV     AL,0xdf         ; enable A20
        OUT     0x60,AL
        CALL    waitkbdout

; プロテクトモード移行

        LGDT    [GDTR0]         ; 暫定GDTを設定
        MOV     EAX,CR0
        AND     EAX,0x7fffffff  ; bit31を0にする（ページング禁止のため）
        OR      EAX,0x00000001  ; bit0を1にする（プロテクトモード移行のため）
        MOV     CR0,EAX
        JMP     pipelineflush
pipelineflush:
        MOV     AX,1*8          ;  読み書き可能セグメント32bit
        MOV     DS,AX
        MOV     ES,AX
        MOV     FS,AX
        MOV     GS,AX
        MOV     SS,AX

; bootpackの転送

        MOV     ESI,bootpack    ; 転送元
        MOV     EDI,BOTPAK      ; 転送先
        MOV     ECX,512*1024/4
        CALL    memcpy

; ついでにディスクデータも本来の位置へ転送

; まずはブートセクタから

        MOV     ESI,0x7c00      ; 転送元
        MOV     EDI,DSKCAC      ; 転送先
        MOV     ECX,512/4
        CALL    memcpy

; 残り全部

        MOV     ESI,DSKCAC0+512 ; 転送元
        MOV     EDI,DSKCAC+512  ; 転送先
        MOV     ECX,0
        MOV     CL,BYTE [CYLS]
        IMUL    ECX,512*18*2/4  ; シリンダ数からバイト数/4に変換
        SUB     ECX,512/4       ; IPLの分だけ差し引く
        CALL    memcpy

; asmheadでしなければいけないことは全部し終わったので、
;   あとはbootpackに任せる

; bootpackの起動

        MOV     EBX,BOTPAK
        MOV     ECX,[EBX+16]
        ADD     ECX,3           ; ECX += 3;
        SHR     ECX,2           ; ECX /= 4;
        JZ      skip            ; 転送するべきものがない
        MOV     ESI,[EBX+20]    ; 転送元
        ADD     ESI,EBX
        MOV     EDI,[EBX+12]    ; 転送先
        CALL    memcpy
skip:
        MOV     ESP,[EBX+12]    ; スタック初期値
        JMP     DWORD 2*8:0x0000001b

waitkbdout:
        IN       AL,0x64
        AND      AL,0x02
        JNZ     waitkbdout      ; ANDの結果が0でなければwaitkbdoutへ
        RET

memcpy:
        MOV     EAX,[ESI]
        ADD     ESI,4
        MOV     [EDI],EAX
        ADD     EDI,4
        SUB     ECX,1
        JNZ     memcpy          ; 引き算した結果が0でなければmemcpyへ
        RET
; memcpyはアドレスサイズプリフィクスを入れ忘れなければ、ストリング命令でも書ける

        ALIGNB  16, DB 0
GDT0:
        TIMES 8 DB 0                ; ヌルセレクタ
        DW      0xffff,0x0000,0x9200,0x00cf ; 読み書き可能セグメント32bit
        DW      0xffff,0x0000,0x9a28,0x0047 ; 実行可能セグメント32bit（bootpack用）

        DW      0
GDTR0:
        DW      8*3-1
        DD      GDT0

        ALIGNB  16
bootpack:

