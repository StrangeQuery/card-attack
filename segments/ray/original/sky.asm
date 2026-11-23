; Sky machine, once per row above the floor (rows 0..21). Deck: the row y. The sky is three bands, rows 0-5, 6-12
; and 13-18; rows 19-21, the floor too close to the horizon to draw, are haze. Punches 96 copies of the row's code.
start:  INP y
        CLA y
        SUB k6
        TAC b0
        CLA y
        SUB k13
        TAC b1
        CLA y
        SUB k19
        TAC b2
        CLA k3
        JMP go
b0:     CLA k0
        JMP go
b1:     CLA k1
        JMP go
b2:     CLA k2
go:     STO v
        CLA kn96
        STO c
lp:     OUT v
        CLA c
        ADD 00
        STO c
        TAC lp
        HRS 0
y:      DAT 0
v:      DAT 0
c:      DAT 0
k0:     DAT 0
k1:     DAT 1
k2:     DAT 2
k3:     DAT 3
k6:     DAT 6
k13:    DAT 13
k19:    DAT 19
kn96:   DAT -96
