; Hit machine, once per ball pixel whose ray reaches the floor. Deck: t (tenths), the ray's x and z (out of 98),
; the pixel's a and c, the ball's x, the floor's scroll in x and z. The floor point is (ball x + a + t x / 10,
; 60 - c + t z / 10), the divisions by a decimal shift. Punches the reflected checker, 10 or 11.
start:  INP t
        INP rx
        INP rz
        INP a
        INP c
        INP bx
        INP xs
        INP zs
        CLA k0           ; t rx and t rz, by adding t times
        STO px
        STO pz
ml:     CLA t
        SUB 00
        TAC md
        STO t
        CLA px
        ADD rx
        STO px
        CLA pz
        ADD rz
        STO pz
        JMP ml
md:     CLA px           ; X + 220 + xs, its tile
        SFT 01
        ADD bx
        ADD a
        ADD k220
        ADD xs
        SFT 01
        STO px
        CLA pz           ; Z + zs, its tile
        SFT 01
        ADD k60
        SUB c
        ADD zs
        SFT 01
        ADD px
pl:     SUB k2
        TAC pd
        JMP pl
pd:     ADD k12
        STO px
        OUT px
        HRS 0
t:      DAT 0
rx:     DAT 0
rz:     DAT 0
a:      DAT 0
c:      DAT 0
bx:     DAT 0
xs:     DAT 0
zs:     DAT 0
px:     DAT 0
pz:     DAT 0
k0:     DAT 0
k2:     DAT 2
k12:    DAT 12
k60:    DAT 60
k220:   DAT 220
