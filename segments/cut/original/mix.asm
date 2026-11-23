; Dissolve machine, once per screen row per frame of a transition. Deck: the level (0..16), the screen row y, then for
; each of the 96 pixels the outgoing picture's red, green and blue and the incoming one's. Where the level beats the
; pixel's place in a 4 x 4 ordered dither the incoming pixel shows, else the outgoing: one dissolves into the other as
; the level rises. Punches the row's 288 values.
start:  INP f
        INP y
        CLA y            ; this row's four dither thresholds start at BAY + 4 (y mod 4)
lm:     SUB k4
        TAC md
        JMP lm
md:     ADD k4
        STO t
        ADD t
        ADD t
        ADD t
        ADD kLd
        STO row
        CLA kn96
        STO c
        CLA k0
        STO col
px:     INP r
        INP g
        INP b
        INP r2
        INP g2
        INP b2
        CLA row          ; CLA BAY + 4 (y mod 4) + x mod 4
        ADD col
        STO get
get:    DAT 0
        SUB f
        TAC in           ; threshold < level
        OUT r
        OUT g
        OUT b
        JMP nx
in:     OUT r2
        OUT g2
        OUT b2
nx:     CLA col
        ADD 00
        STO col
        SUB k4
        TAC nd
        CLA k0
        STO col
nd:     CLA c
        ADD 00
        STO c
        TAC px
        HRS 0
f:      DAT 0
y:      DAT 0
t:      DAT 0
row:    DAT 0
c:      DAT 0
col:    DAT 0
r:      DAT 0
g:      DAT 0
b:      DAT 0
r2:     DAT 0
g2:     DAT 0
b2:     DAT 0
k0:     DAT 0
k4:     DAT 4
kn96:   DAT -96
kLd:    DAT BAY+100
BAY:    DAT 0
        DAT 8
        DAT 2
        DAT 10
        DAT 12
        DAT 4
        DAT 14
        DAT 6
        DAT 3
        DAT 11
        DAT 1
        DAT 9
        DAT 15
        DAT 7
        DAT 13
        DAT 5
