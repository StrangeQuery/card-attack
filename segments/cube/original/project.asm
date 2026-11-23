; Projector. Deck: the centre (cx, cy), the tilt T (0-3), then 8 corners as (x, y, depth). Depth shifts a corner
; up and to the right by a = trunc(T depth / 10), so we see its front, top and right side: punches
; screen x = cx + x + a and screen y = cy + y - a. T = 0 is the flat square.
start:  INP cx
        INP cy
        INP T
        CLA kn8
        STO vc
vl:     INP X
        INP Y
        INP D
        CLA k0           ; a = T depth, by adding
        STO a
        CLA T
        STO m
ml:     CLA m
        SUB 00
        TAC md
        STO m
        CLA a
        ADD D
        STO a
        JMP ml
md:     CLA a            ; / 10
        SFT 01
        STO a
        CLA cx
        ADD X
        ADD a
        STO X
        OUT X
        CLA cy
        ADD Y
        SUB a
        STO Y
        OUT Y
        CLA vc
        ADD 00
        STO vc
        TAC vl
        HRS 0
cx:     DAT 0
cy:     DAT 0
T:      DAT 0
X:      DAT 0
Y:      DAT 0
D:      DAT 0
a:      DAT 0
m:      DAT 0
vc:     DAT 0
k0:     DAT 0
kn8:    DAT -8
