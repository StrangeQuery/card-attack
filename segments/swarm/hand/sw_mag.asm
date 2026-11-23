; sw_mag by hand: signs and sizes. Deck as sw_mag's: the count; then each: how many cards (1 or 3) and the cards
; (three: millions, thousands, rest, the last two 0..999). Punches 1 if it is below 0 (else 0), then its size's
; three cards.
; Against the compiled sw_mag: a one-card number never built into three cards first (below 0, its size is just
; 0 - h0), and each case straight to its own punches.
start:  INP n
item:   CLA n
        SUB 00
        TAC done
        STO n
        INP k
        CLA k
        SUB 00
        SUB 00
        TAC one
        INP h2           ; three cards
        INP h1
        INP h0
        CLA h2
        TAC neg
        OUT k0
        OUT h2
        OUT h1
        OUT h0
        JMP item
neg:    CLA k0           ; below 0: each card's complement, the borrow running up from the rest
        SUB h2
        STO h2
        CLA h0
        SUB 00
        TAC z0
        CLA k999         ; the rest above 0: 1000 - h0, 999 - h1, h2 - 1
        SUB h0
        ADD 00
        STO h0
        CLA k999
        SUB h1
        STO h1
        JMP dec
z0:     CLA h1           ; the rest 0: the thousands decide
        SUB 00
        TAC put
        CLA k999
        SUB h1
        ADD 00
        STO h1
dec:    CLA h2
        SUB 00
        STO h2
put:    OUT 00
        OUT h2
        OUT h1
        OUT h0
        JMP item
one:    INP h0           ; one card
        CLA h0
        TAC neg1
        OUT k0
        OUT k0
        OUT k0
        OUT h0
        JMP item
neg1:   CLA k0
        SUB h0
        STO h0
        OUT 00
        OUT k0
        OUT k0
        OUT h0
        JMP item
done:   HRS 0
n:      DAT 0
k:      DAT 0
h2:     DAT 0
h1:     DAT 0
h0:     DAT 0
k0:     DAT 0
k999:   DAT 999
