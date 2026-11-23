; Floor machine, once per floor row per frame. Deck: from the row machine, Z, the checker-row parity, the starting x
; and its remainder, and (Z - 60)^2 or -1; then the row's d, the floor's scroll in x (0..19) and the ball's x.
; Steps across the row, x = floor((col - 48) 16 / d), by adding 16 to the remainder each column. Each pixel is
; checker 4 or 5 by the parity of (x + 200 + scroll) / 10 plus the row's, 2 darker inside the ball's shadow, a
; circle of radius 12 around (ball x, 60). Punches 96 colour codes.
start:  INP z
        INP zp
        INP x
        INP rm
        INP dz2
        INP d
        INP xs
        INP bx
        CLA kn96
        STO c
px:     CLA x            ; checker: parity of (x + 200 + xs) / 10 + zp
        ADD k200
        ADD xs
        SFT 01
        ADD zp
pl:     SUB k2
        TAC pd
        JMP pl
pd:     ADD k6           ; 4 or 5
        STO v
        CLA dz2          ; shadow
        TAC ns
        CLA x
        SUB bx
        TAC xn
        JMP xp
xn:     STO t
        CLA k0
        SUB t
xp:     STO t
        SUB k12
        TAC sq
        JMP ns
sq:     CLA t            ; dx^2 + dz^2 < 144
        STO i
        CLA dz2
        STO s
s3:     CLA i
        SUB 00
        TAC s4
        STO i
        CLA s
        ADD t
        STO s
        JMP s3
s4:     CLA s
        SUB k144
        TAC sh
        JMP ns
sh:     CLA v
        ADD k2
        STO v
ns:     OUT v
        CLA rm           ; next column
        ADD k16
st:     STO rm
        SUB d
        TAC nx
        STO rm
        CLA x
        ADD 00
        STO x
        CLA rm
        JMP st
nx:     CLA c
        ADD 00
        STO c
        TAC px
        HRS 0
z:      DAT 0
zp:     DAT 0
x:      DAT 0
rm:     DAT 0
d:      DAT 0
xs:     DAT 0
bx:     DAT 0
t:      DAT 0
i:      DAT 0
s:      DAT 0
dz2:    DAT 0
c:      DAT 0
v:      DAT 0
k0:     DAT 0
k2:     DAT 2
k6:     DAT 6
k12:    DAT 12
k16:    DAT 16
k60:    DAT 60
k144:   DAT 144
k200:   DAT 200
kn96:   DAT -96
