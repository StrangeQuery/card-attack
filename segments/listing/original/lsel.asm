; Listing row selector, once per screen row per frame. Deck: the scroll s (pixels) and the screen row y. The row
; shows listing pixel row s + y: punches the listing line (s + y) / 6 and the glyph row (s + y) mod 6.
start:  INP s
        INP y
        CLA s
        ADD y
        STO t
        CLA k0
        STO L
lp:     CLA t
        SUB k6
        TAC dn
        STO t
        CLA L
        ADD 00
        STO L
        JMP lp
dn:     OUT L
        OUT t
        HRS 0
s:      DAT 0
y:      DAT 0
t:      DAT 0
L:      DAT 0
k0:     DAT 0
k6:     DAT 6
