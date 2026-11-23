; Compose machine, once per row per frame. Deck: where the ball's span starts and how many pixels it covers, the
; ball's colour codes for those pixels, then the row's 96 background codes. Punches the row with the ball over it.
start:  INP x0
        INP n
        CLA kIn          ; read the ball's codes into BALL
        STO rd
        CLA k0
        SUB n
        STO c
rl:     CLA c
        TAC rd
        JMP go
rd:     DAT 0
        CLA rd
        ADD 00
        STO rd
        CLA c
        ADD 00
        STO c
        JMP rl
go:     CLA kOut
        STO wr
        CLA kn96
        STO c
px:     INP v
        CLA x0           ; before the span: count down
        SUB 00
        TAC sp
        STO x0
        JMP bg
sp:     CLA n            ; in the span while any ball codes are left
        SUB 00
        TAC bg
        STO n
wr:     DAT 0            ; OUT BALL + i
        CLA wr
        ADD 00
        STO wr
        JMP nx
bg:     OUT v
nx:     CLA c
        ADD 00
        STO c
        TAC px
        HRS 0
x0:     DAT 0
n:      DAT 0
c:      DAT 0
v:      DAT 0
k0:     DAT 0
kn96:   DAT -96
kIn:    DAT BALL
kOut:   DAT BALL+500
BALL:   DAT 0
