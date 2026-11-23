; Palette. Deck: for each face, its brightness v (0-16) and its colour (r, g, b) in sixteenths. Punches the lit
; colour: each channel times v, by adding.
start:  CLA kn6
        STO fc
fl:     INP v
        CLA kn3
        STO cc
cl:     INP b
        CLA k0
        STO r
        CLA v
        STO m
ml:     CLA m
        SUB 00
        TAC md
        STO m
        CLA r
        ADD b
        STO r
        JMP ml
md:     OUT r
        CLA cc
        ADD 00
        STO cc
        TAC cl
        CLA fc
        ADD 00
        STO fc
        TAC fl
        HRS 0
v:      DAT 0
b:      DAT 0
r:      DAT 0
m:      DAT 0
cc:     DAT 0
fc:     DAT 0
k0:     DAT 0
kn3:    DAT -3
kn6:    DAT -6
