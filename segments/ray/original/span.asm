; Span machine, once per row per frame. Deck: the row y and the ball's x and height. The ball, radius 14 on screen,
; is centred at column 48 + x and row 34 - height. Punches where the ball's span starts on this row and how many
; pixels it covers: those with a^2 + b^2 < 196 (0, 0 if none); then b, the row's height above the centre, and
; each covered pixel's a, its offset across from the centre.
start:  INP y
        INP bx
        INP by
        CLA k34          ; b = 34 - height - y
        SUB by
        SUB y
        TAC bn
        JMP bp
bn:     STO b
        CLA k0
        SUB b
bp:     STO b            ; |b|
        STO i
        SUB k14
        TAC in
        OUT k0
        OUT k0
        HRS 0
in:     CLA k195         ; n = 195 - b^2
        STO n
l1:     CLA i
        SUB 00
        TAC sq
        STO i
        CLA n
        SUB b
        STO n
        JMP l1
sq:     CLA k0           ; hw = isqrt(n)
        STO w
        CLA k1
        STO od
ql:     CLA n
        SUB od
        TAC qd
        STO n
        CLA od
        ADD k2
        STO od
        CLA w
        ADD 00
        STO w
        JMP ql
qd:     CLA k48          ; start 48 + x - hw, count 2 hw + 1
        ADD bx
        SUB w
        STO n
        OUT n
        CLA w
        ADD w
        ADD 00
        STO n
        OUT n
        CLA k34          ; b again, signed
        SUB by
        SUB y
        STO i
        OUT i
        CLA k0           ; a from -hw to hw
        SUB w
        STO i
al:     OUT i
        CLA i
        SUB w
        TAC an
        HRS 0
an:     CLA i
        ADD 00
        STO i
        JMP al
y:      DAT 0
bx:     DAT 0
by:     DAT 0
b:      DAT 0
i:      DAT 0
n:      DAT 0
w:      DAT 0
od:     DAT 0
k0:     DAT 0
k1:     DAT 1
k2:     DAT 2
k14:    DAT 14
k34:    DAT 34
k48:    DAT 48
k195:   DAT 195
