    ; 定義
    CYLS  EQU 10

    ; このプログラムが読み込まれる場所の指定
    ORG   0x7c00

    ; FAT12のための記述
    JMP   entry
    DB    0x90
    DB    "HELLOIPL"                ; ブートセクタの名前
    DW    512                       ; 1セクタの大きさ
    DB    1                         ; クラスタの大きさ
    DW    1                         ; FATの開始地点
    DB    2                         ; FATの個数
    DW    224                       ; ルートディレクトリ領域の大きさ
    DW    2880                      ; このドライブの大きさ
    DB    0xf0                      ; メディアのタイプ
    DW    9                         ; FAT領域の長さ
    DW    18                        ; 1トラックあたりのセクタの数
    DW    2                         ; ヘッドの数
    DD    0                         ; パーティション未使用
    DD    2880                      ; このドライブの大きさ
    DB    0,0,0x29
    DD    0xffffffff                ; ボリュームシリアル番号
    DB    "MY-OS      "
    DB    "FAT12   "
    TIMES 18 DB 0

    ; プログラム本体
entry:

    ; レジスタ初期化
    MOV   AX,0
    MOV   SS,AX
    MOV   SP,0x7c00
    MOV   DS,AX

    ; ディスクの読み込み
    MOV   AX,0x0820
    MOV   ES,AX
    MOV   CH,0                      ; シリンダ0
    MOV   DH,0                      ; ヘッド0
    MOV   CL,2                      ; セクタ0

readloop:
    MOV   SI,0                      ; 失敗回数をカウントするレジスタ

retry:
    MOV   AH,0x02                   ; ディスク読み込み
    MOV   AL,1                      ; 1セクタ分
    MOV   BX,0
    MOV   DL,0x00                   ; Aドライブ
    INT   0x13
    JNC   next                      ; 読み込みに成功したらfinへ
    ADD   SI,1
    CMP   SI,5                      ; 5回失敗したらload error
    JAE   error
    MOV   AH,0x00
    MOV   DL,0x00
    INT   0x13                      ; ドライブリセット
    JMP   retry

next:
    MOV   AX,ES
    ADD   AX,0x0020
    MOV   ES,AX
    ADD   CL,1
    CMP   CL,18
    JBE   readloop                  ; CL <= 18ならジャンプ
    MOV   CL,1
    ADD   DH,1
    CMP   DH,2
    JB    readloop                  ; DH < 2ならジャンプ
    MOV   DH,0
    ADD   CH,1
    CMP   CH,CYLS
    JB    readloop

    MOV   [0x0ff0],CH
    JMP   0xc200

fin:
    HLT
    JMP   fin

error:

    MOV   SI,msg

putloop:
    MOV   AL,[SI]
    ADD   SI,1
    CMP   AL,0
    JE    fin
    MOV   AH,0x0e
    MOV   BX,15
    INT   0x10
    JMP   putloop

msg:
    DB    0x0a, 0x0a
    DB    "load error"
    DB    0x0a
    DB    0
    TIMES 0x7dfe-0x7c00-($-$$) DB 0
    DB    0x55, 0xaa
