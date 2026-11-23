; Listing row machine, once per screen row per frame. Its program deck carries the row of the listing it shows, 24
; glyph rows (three binary digits each) at TEX. Deck: the text colour code. Each character is 4 pixels, 3 of glyph
; and a gap. Punches 96 codes: the colour where a glyph bit is set, else 0.
start:  INP k
        CLA kLd
        STO get
        CLA kn24
        STO c
ch:     CLA k0           ; column 0..2 of this character, then the gap
        STO col
cl:     CLA col
        SFT 10
        ADD kSft
        STO sh
get:    DAT 0            ; CLA TEX + character
sh:     DAT 0            ; SFT 13, 23 or 33: that column's bit
        SUB 00
        TAC z
        OUT k
        JMP nc
z:      OUT k0
nc:     CLA col
        ADD 00
        STO col
        SUB k3
        TAC cl
        OUT k0           ; the gap
        CLA get
        ADD 00
        STO get
        CLA c
        ADD 00
        STO c
        TAC ch
        HRS 0
k:      DAT 0
c:      DAT 0
col:    DAT 0
k0:     DAT 0
k3:     DAT 3
kn24:   DAT -24
kSft:   DAT 413
kLd:    DAT TEX+100
TEX:    DAT 0
