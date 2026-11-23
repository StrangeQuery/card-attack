; Caption machine, once per screen row of a caption's box (rows 0 to 6) per frame. Deck: the fade (0..16), the
; screen row y, the box's right edge x1, then for each of the 96 pixels the caption's text code (1 where a glyph bit
; is set, else 0) and the effect's red, green and blue. Right of x1 the effect shows. Up to x1, where the fade beats
; the pixel's place in a 4 x 4 ordered dither, the box shows: white text on black. Punches the row's 288 values.
start:  INP f
        INP y
        INP x1
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
        CLA x1
        STO dx           ; x1 - x
px:     INP t
        INP r
        INP g
        INP b
        CLA dx
        TAC pass         ; right of the box
        CLA row          ; CLA BAY + 4 (y mod 4) + x mod 4
        ADD col
        STO get
get:    DAT 0
        SUB f
        TAC on           ; threshold < fade
pass:   OUT r
        OUT g
        OUT b
        JMP nx
on:     CLA t
        SUB 00
        TAC blk
        OUT kR
        OUT kG
        OUT kB
        JMP nx
blk:    OUT k0
        OUT k0
        OUT k0
nx:     CLA dx
        SUB 00
        STO dx
        CLA col
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
x1:     DAT 0
t:      DAT 0
r:      DAT 0
g:      DAT 0
b:      DAT 0
row:    DAT 0
c:      DAT 0
col:    DAT 0
dx:     DAT 0
k0:     DAT 0
k4:     DAT 4
kn96:   DAT -96
kR:     DAT 240
kG:     DAT 240
kB:     DAT 240
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
