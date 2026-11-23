; sw_lsum by hand: long sums for the swarm's arithmetic. Deck as sw_lsum's: the count; then each sum: how many
; terms, and each term's three cards, millions, thousands and the rest (the last two 0..999). Punches each sum the
; same way.
; Against the compiled sw_lsum: the first term loaded straight in (from 0 there's nothing to carry), each card
; stored before its carry test, the units' carry one more on the thousands' add (t1 + 1 + h1 carries at most once,
; where the compiled code tests h1 == 999 apart), the millions' carry in the accumulator.
start:  INP n
sum:    CLA n
        SUB 00
        TAC done
        STO n
        INP j
        CLA j
        SUB 00
        TAC zero
        STO j
        INP h2
        INP h1
        INP h0
term:   CLA j
        SUB 00
        TAC put
        STO j
        INP t2
        INP t1
        INP t0
        CLA h0           ; the units: h0 + t0, and whether they carry
        ADD t0
        STO h0
        SUB k999
        SUB 00
        TAC c0n
        STO h0
        CLA h1           ; carried: the thousands get t1 + 1
        ADD t1
        ADD 00
        JMP c1t
c0n:    CLA h1
        ADD t1
c1t:    STO h1
        SUB k999
        SUB 00
        TAC c1n
        STO h1           ; the thousands carry: the millions get t2 + 1
        CLA h2
        ADD t2
        ADD 00
        STO h2
        JMP term
c1n:    CLA h2
        ADD t2
        STO h2
        JMP term
put:    OUT h2
        OUT h1
        OUT h0
        JMP sum
zero:   OUT k0           ; no terms: 0
        OUT k0
        OUT k0
        JMP sum
done:   HRS 0
n:      DAT 0
j:      DAT 0
h2:     DAT 0
h1:     DAT 0
h0:     DAT 0
t2:     DAT 0
t1:     DAT 0
t0:     DAT 0
k0:     DAT 0
k999:   DAT 999
