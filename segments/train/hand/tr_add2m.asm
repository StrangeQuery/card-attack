; Adding longs by hand, two longs and the constant, the second taken away (k = 2, sign -1). Deck (the three's): the
; count, k, the sign, the constant's two cards (hi, lo); then each item's two longs. Punches each a + c - b as a long.
; The high once, after the carry and the borrow are known (hi + ch - bh, plus 1, less 1, or both cancelling), in the
; accumulator: never a partial high on a card.
start:  INP n
        INP k
        INP s2
        INP ch
        INP m
        CLA m            ; m = cl - 999
        SUB k999
        STO m
loop:   CLA n
        SUB 00
        TAC done
        STO n
        INP hi
        INP lo
        INP bh
        INP bl
        CLA lo
        ADD m
        SUB 00
        TAC n1           ; lo + cl - 1000 below 0: no carry
        SUB bl           ; the carry: lo1 = lo + cl - 1000; lo1 - bl
        TAC c1b
        STO lo           ; carry, no borrow: hi + ch - bh + 1
        CLA hi
        ADD ch
        SUB bh
        ADD 00
        STO hi
        OUT hi
        OUT lo
        JMP loop
c1b:    ADD k999         ; carry and borrow: lo1 - bl + 1000, hi + ch - bh
        ADD 00
        STO lo
        CLA hi
        ADD ch
        SUB bh
        STO hi
        OUT hi
        OUT lo
        JMP loop
n1:     ADD k999         ; no carry: lo1 = lo + cl; lo1 - bl
        ADD 00
        SUB bl
        TAC n1b
        STO lo
        CLA hi
        ADD ch
        SUB bh
        STO hi
        OUT hi
        OUT lo
        JMP loop
n1b:    ADD k999         ; the borrow only: lo1 - bl + 1000, hi + ch - bh - 1
        ADD 00
        STO lo
        CLA hi
        ADD ch
        SUB bh
        SUB 00
        STO hi
        OUT hi
        OUT lo
        JMP loop
done:   HRS 0
n:      DAT 0
k:      DAT 0
s2:     DAT 0
ch:     DAT 0
m:      DAT 0
hi:     DAT 0
lo:     DAT 0
bh:     DAT 0
bl:     DAT 0
k999:   DAT 999
