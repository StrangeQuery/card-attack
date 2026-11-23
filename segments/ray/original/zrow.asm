; Floor row machine, once per floor row per frame. Deck: the row's distance below the horizon d (4..29), the floor's
; scroll in z (0..19), the ball's x. The floor is 16 units below the eye and the screen is 60 units away, so the
; row shows the floor at Z = 960 / d. Punches Z, the parity of its checker row (Z + scroll) / 10, the starting x,
; floor(-768 / d), with its remainder, and (Z - 60)^2 if the ball's shadow (radius 12 around Z = 60) can reach
; this row, else -1.
start:  INP d
        INP zs
        CLA k960         ; Z = 960 / d (960 as two cards: 480 twice)
        ADD k480
        STO n
        CLA k0
        STO z
zl:     CLA n
        SUB d
        TAC zd
        STO n
        CLA z
        ADD 00
        STO z
        JMP zl
zd:     OUT z
        CLA z            ; parity of (Z + zs) / 10
        ADD zs
        SFT 01
        STO n
pl:     CLA n
        SUB k2
        TAC pd
        STO n
        JMP pl
pd:     OUT n
        CLA k768         ; q = 768 / d, then x0 = -q, or -q - 1 with remainder d - r if r > 0
        STO n
        CLA k0
        STO q
ql:     CLA n
        SUB d
        TAC qd
        STO n
        CLA q
        ADD 00
        STO q
        JMP ql
qd:     CLA n
        SUB 00
        TAC ex
        CLA d
        SUB n
        STO n
        CLA q
        ADD 00
        STO q
ex:     CLA k0
        SUB q
        STO q
        OUT q
        OUT n
        CLA z            ; shadow possible: |Z - 60| < 12
        SUB k60
        TAC sn
        JMP sp
sn:     STO t
        CLA k0
        SUB t
sp:     STO t
        SUB k12
        TAC sq
        OUT kn1
        HRS 0
sq:     CLA t            ; (Z - 60)^2
        STO q
        CLA k0
        STO n
s1:     CLA q
        SUB 00
        TAC s2
        STO q
        CLA n
        ADD t
        STO n
        JMP s1
s2:     OUT n
        HRS 0
d:      DAT 0
zs:     DAT 0
n:      DAT 0
z:      DAT 0
q:      DAT 0
k0:     DAT 0
kn1:    DAT -1
t:      DAT 0
k2:     DAT 2
k12:    DAT 12
k60:    DAT 60
k480:   DAT 480
k768:   DAT 768
k960:   DAT 480
