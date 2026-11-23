; tr_sum by hand: sums of longs, each term added or taken away. Deck as tr_sum's: the count, how many terms; then
; each sum's terms: the sign (1 or -1), the long (hi, lo). Punches each sum as a long (hi, lo), lo 0..999.
; Against the compiled tr_sum (27-28 instructions an added term, 20-26 a taken one): the sign tested by TAC alone
; (-1 is negative), the low stored before the carry test so no carry leaves it as it is, the carry added to the
; high in the accumulator (never a partial high on a card), the terms counted down, and the first term loaded
; straight in (from 0 there's nothing to carry).
start:  INP n
        INP w
sum:    CLA n
        SUB 00
        TAC done
        STO n
        CLA w            ; the first term straight in: from 0 there's no carry to test (w is 1 or more)
        SUB 00
        STO j
        INP s
        INP th
        INP tl
        CLA s
        TAC fneg
        CLA th           ; added: hi, lo as they are
        STO hi
        CLA tl
        STO lo
        JMP term
fneg:   CLA k0           ; taken away: 0 - tl, borrowing unless tl is 0
        SUB tl
        TAC fb
        STO lo
        CLA k0
        SUB th
        STO hi
        JMP term
fb:     ADD k999         ; 1000 - tl, and -th - 1
        ADD 00
        STO lo
        CLA k0
        SUB th
        SUB 00
        STO hi
        JMP term
term:   CLA j
        SUB 00
        TAC put
        STO j
        INP s
        INP th
        INP tl
        CLA s
        TAC take
        CLA lo           ; added: lo + tl, and whether it passes 999
        ADD tl
        STO lo
        SUB k999
        SUB 00
        TAC nc
        STO lo           ; the carry: lo + tl - 1000, hi + th + 1
        CLA hi
        ADD th
        ADD 00
        STO hi
        JMP term
nc:     CLA hi           ; no carry: hi + th
        ADD th
        STO hi
        JMP term
take:   CLA lo           ; taken away: lo - tl, and whether it borrows
        SUB tl
        TAC b
        STO lo
        CLA hi
        SUB th
        STO hi
        JMP term
b:      ADD k999         ; the borrow: lo - tl + 1000, hi - th - 1
        ADD 00
        STO lo
        CLA hi
        SUB th
        SUB 00
        STO hi
        JMP term
put:    OUT hi
        OUT lo
        JMP sum
done:   HRS 0
n:      DAT 0
w:      DAT 0
j:      DAT 0
s:      DAT 0
th:     DAT 0
tl:     DAT 0
hi:     DAT 0
lo:     DAT 0
k0:     DAT 0
k999:   DAT 999
