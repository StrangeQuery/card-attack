; Typesetter, once per listing line per glyph row. Deck: the glyph row g (0..4) and the number of characters n, then for each
; character its five font cards (each a glyph row as three binary digits, 101 for X.X). Punches the g-th font card
; of each character: one row of the line, n cards.
start:  INP g
        INP n
        CLA k0
        SUB n
        STO c
ch:     CLA k0           ; read five cards, keep the g-th
        STO i
rd:     INP v
        CLA i
        SUB g
        TAC sk
        SUB 00
        TAC kp
        JMP sk
kp:     CLA v
        STO w
sk:     CLA i
        ADD 00
        STO i
        SUB k5
        TAC rd
        OUT w
        CLA c
        ADD 00
        STO c
        TAC ch
        HRS 0
g:      DAT 0
c:      DAT 0
i:      DAT 0
v:      DAT 0
w:      DAT 0
n:      DAT 0
k0:     DAT 0
k5:     DAT 5
