; Gather: picks corners out of the projected list. Deck: the 8 projected corners (x, y), then corner numbers,
; three per face. Punches each numbered corner's (x, y).
start:  CLA inP          ; read the 16 coordinates into P
        STO rd
        CLA kn16
        STO c
rd:     DAT 0            ; becomes INP P+i
        CLA rd
        ADD 00
        STO rd
        CLA c
        ADD 00
        STO c
        TAC rd
        CLA kn18
        STO c
gl:     INP i
        CLA i            ; OUT P+2i, OUT P+2i+1
        ADD i
        ADD outP
        STO o1
        ADD 00
        STO o2
o1:     DAT 0
o2:     DAT 0
        CLA c
        ADD 00
        STO c
        TAC gl
        HRS 0
i:      DAT 0
c:      DAT 0
kn16:   DAT -16
kn18:   DAT -18
inP:    DAT P            ; INP P
outP:   DAT P+500        ; OUT P
P:      DAT 0
