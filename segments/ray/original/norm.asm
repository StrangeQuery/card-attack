; Normal machine, once per ball pixel. Deck: the pixel's offset from the ball's centre, a across and b up. The
; surface faces the camera by c = isqrt(196 - a^2 - b^2). A ray from the eye bounces off it in the direction
; (c a, c b, 98 - c^2), out of 98. Punches c and that direction.
start:  INP a
        INP b
        CLA k196         ; n = 196 - a^2 - b^2, by adding |a| |a| times and |b| |b| times
        STO n
        CLA a
        TAC an
        JMP ap
an:     STO t
        CLA k0
        SUB t
ap:     STO t
        STO i
l1:     CLA i
        SUB 00
        TAC lb
        STO i
        CLA n
        SUB t
        STO n
        JMP l1
lb:     CLA b
        TAC bn
        JMP bp
bn:     STO t
        CLA k0
        SUB t
bp:     STO t
        STO i
l2:     CLA i
        SUB 00
        TAC sq
        STO i
        CLA n
        SUB t
        STO n
        JMP l2
sq:     CLA k0           ; c = isqrt(n)
        STO c
        CLA k1
        STO od
ql:     CLA n
        SUB od
        TAC qd
        STO n
        CLA od
        ADD k2
        STO od
        CLA c
        ADD 00
        STO c
        JMP ql
qd:     OUT c
        CLA c            ; c a, c b, c c by adding c times
        STO i
        CLA k0
        STO ra
        STO rb
        STO rc
ml:     CLA i
        SUB 00
        TAC md
        STO i
        CLA ra
        ADD a
        STO ra
        CLA rb
        ADD b
        STO rb
        CLA rc
        ADD c
        STO rc
        JMP ml
md:     OUT ra
        OUT rb
        CLA k98
        SUB rc
        STO rc
        OUT rc
        HRS 0
a:      DAT 0
b:      DAT 0
n:      DAT 0
t:      DAT 0
i:      DAT 0
c:      DAT 0
od:     DAT 0
ra:     DAT 0
rb:     DAT 0
rc:     DAT 0
k0:     DAT 0
k1:     DAT 1
k2:     DAT 2
k98:    DAT 98
k196:   DAT 196
