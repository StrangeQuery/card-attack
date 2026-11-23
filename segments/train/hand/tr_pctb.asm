; tr_pctb by hand: shares and mixes' second half: the product from its two quarter squares (longs), over 100 cut
; toward zero, added to a. Deck as tr_pctb's: the count, then each: the two quarter squares and a. Punches each
; result (a card).
; By shifts: p / 100 is hi 10 + lo / 100 (SFT 10, SFT 13), one more when hi is below 0 and lo / 100 left a rest
; (SFT 22 is lo's last two digits).
start:  INP n
item:   CLA n
        SUB 00
        TAC done
        STO n
        INP hi
        INP lo
        INP bh
        INP bl
        INP a
        CLA lo
        SUB bl
        TAC bw
        STO lo
        CLA hi
        SUB bh
        JMP h1
bw:     ADD k999
        ADD 00
        STO lo
        CLA hi
        SUB bh
        SUB 00
h1:     TAC neg
        SFT 10           ; hi 10 + lo / 100 + a
        STO t
        CLA lo
        SFT 13
        ADD t
        ADD a
        STO t
        OUT t
        JMP item
neg:    SFT 10
        STO t
        CLA lo
        SFT 22
        SUB 00
        TAC even
        CLA lo           ; a rest: one more, toward zero
        SFT 13
        ADD t
        ADD 00
        ADD a
        STO t
        OUT t
        JMP item
even:   CLA lo
        SFT 13
        ADD t
        ADD a
        STO t
        OUT t
        JMP item
done:   HRS 0
n:      DAT 0
hi:     DAT 0
lo:     DAT 0
bh:     DAT 0
bl:     DAT 0
a:      DAT 0
t:      DAT 0
k999:   DAT 999
