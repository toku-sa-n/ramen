; 定義
cyls    equ 10

; このプログラムが読み込まれる場所の指定
    org     0x7c00

; FAT12のための記述
    jmp     entry
    db      0x90
    db      "HELLOIPL"  ; ブートセクタの名前
    dw      512         ; 1セクタの大きさ
    db      1           ; クラスタの大きさ
    dw      1           ; FATの開始地点
    db      2           ; FATの個数
    dw      224         ; ルートディレクトリ領域の大きさ
    dw      2880        ; このドライブの大きさ
    db      0xf0        ; メディアのタイプ
    dw      9           ; FAT領域の長さ
    dw      18          ; 1トラックあたりのセクタの数
    dw      2           ; ヘッドの数
    dd      0           ; パーティション未使用
    dd      2880        ; このドライブの大きさ
    db      0,0,0x29
    dd      0xffffffff  ; ボリュームシリアル番号
    db      "MY-OS      "
    db      "FAT12   "
    times 18 db 0

; プログラム本体
entry:

; レジスタ初期化
    mov     ax,0
    mov     ss,ax
    mov     sp,0x7c00
    mov     ds,ax

; ディスクの読み込み
    mov     ax,0x0820
    mov     es,ax
    mov     ch,0        ; シリンダ0
    mov     dh,0        ; ヘッド0
    mov     cl,2        ; セクタ0

readloop:
    mov     si,0        ; 失敗回数をカウントするレジスタ

retry:
    mov     ah,0x02     ; ディスク読み込み
    mov     al,1        ; 1セクタ分
    mov     bx,0
    mov     dl,0x00     ; Aドライブ
    int     0x13
    jnc     next        ; 読み込みに成功したらfinへ
    add     si,1
    cmp     si,5        ; 5回失敗したらload error
    jae     error
    mov     ah,0x00
    mov     dl,0x00
    int     0x13        ; ドライブリセット
    jmp     retry

next:
    mov     ax,es
    add     ax,0x0020
    mov     es,ax
    add     cl,1
    cmp     cl,18
    jbe     readloop    ; CL <= 18ならジャンプ
    mov     cl,1
    add     dh,1
    cmp     dh,2
    jb      readloop    ; DH < 2ならジャンプ
    mov     dh,0
    add     ch,1
    cmp     ch,cyls
    jb      readloop

    mov     [0x0ff0],ch
    jmp     0xc200

fin:
    hlt
    jmp     fin

error:

    mov     si,msg

putloop:
    mov     al,[si]
    add     si,1
    cmp     al,0
    je      fin
    mov     ah,0x0e
    mov     bx,15
    int     0x10
    jmp     putloop

msg:
    db      0x0a, 0x0a
    db      "load error"
    db      0x0a
    db      0
    times   0x7dfe-0x7c00-($-$$) db 0
    db      0x55, 0xaa
