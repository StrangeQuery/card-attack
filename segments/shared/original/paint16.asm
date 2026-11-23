; Paint. Deck: a 16-colour palette (r, g, b each), then one code per pixel for this row. Punches each pixel's colour.
start:  CLA inP          ; read the 48 palette cards into PAL
        STO rd
        CLA kn48
        STO c
rd:     DAT 0            ; becomes INP PAL+i
        CLA rd
        ADD 00
        STO rd
        CLA c
        ADD 00
        STO c
        TAC rd
        CLA kn96
        STO c
px:     INP code
        CLA code         ; OUT PAL+3code, +1, +2
        ADD code
        ADD code
        ADD outP
        STO o1
        ADD 00
        STO o2
        ADD 00
        STO o3
o1:     DAT 0
o2:     DAT 0
o3:     DAT 0
        CLA c
        ADD 00
        STO c
        TAC px
        HRS 0
code:   DAT 0
c:      DAT 0
kn48:   DAT -48
kn96:   DAT -96
inP:    DAT PAL
outP:   DAT PAL+500
PAL:    DAT 0
