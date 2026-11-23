; Big word machine, once per screen row per frame a word is up. Deck: the fade (0..16), the screen row y, then for
; each of the 96 pixels the word's bitmap code (0 none, 1 letter, 2 its black rim) and the picture's red, green and
; blue. Where the word has something and the fade beats the pixel's place in a 4 x 4 ordered dither, the word shows:
; white letters rimmed in black. Punches the row's 288 values.
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
px:     INP t
        INP r
        INP g
        INP b
        CLA t
        SUB 00
        TAC pass         ; nothing of the word here
        CLA row          ; CLA BAY + 4 (y mod 4) + x mod 4
        ADD col
        STO get
get:    DAT 0
        SUB f
        TAC on
pass:   OUT r
        OUT g
        OUT b
        JMP nx
on:     CLA t
        SUB k2
        TAC wh
        OUT k0           ; the rim
        OUT k0
        OUT k0
        JMP nx
wh:     OUT kW
        OUT kW
        OUT kW
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
r:      DAT 0
g:      DAT 0
b:      DAT 0
row:    DAT 0
c:      DAT 0
col:    DAT 0
k0:     DAT 0
k2:     DAT 2
k4:     DAT 4
kn96:   DAT -96
kW:     DAT 250
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
