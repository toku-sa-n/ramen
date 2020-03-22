    [BITS 64]
    BOTPAK   EQU     0x00200000          ; bootpackのロード先

    ; BOOT_INFO関係
    CYLS     EQU     0x0FF0              ; ブートセクタが設定する
    LEDS     EQU     0x0FF1

    ORG      0x500

    ; PICが一切の割り込みを受け付けないようにする
    ; AT互換機の仕様では、PICの初期化をするなら、
    ; こいつをCLI前にやっておかないと、たまにハングアップする
    ; PICの初期化はあとでやる

    MOV      AL,0xFF
    OUT      0x21,AL
    NOP                                  ; OUT命令を連続させるとうまくいかない機種があるらしいので
    OUT      0xA1,AL

    CLI                                  ; さらにCPUレベルでも割り込み禁止

    ; bootpackの起動

    %include "paging_64.asm"

    MOV      RSP,0xFFFFFFFF800a1000                ; スタック初期値

    ; JMP 0xFFFFFFFF80000000 can't be executed.
    ; Jump to 64 bit immediate address is not supported.
    MOV      RDI,0xFFFFFFFF80000000
    JMP      RDI
