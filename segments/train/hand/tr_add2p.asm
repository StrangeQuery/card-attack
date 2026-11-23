; Adding longs by hand, two longs and the constant, the second added (k = 2, sign 1). Deck (the three's): the count,
; k, the sign, the constant's two cards (hi, lo); then each item's two longs (hi, lo, hi, lo). Punches each sum as a
; long. The high is summed once, after both carries are known (hi + ch + bh + carries, in the accumulator, never a
; partial high on a card); with no first carry the accumulator already holds lo + cl - 1000, so the second test is one
; ADD away.
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
        TAC n1           ; lo + cl - 1000 below 0: no first carry
        ADD bl           ; the first carry: the accumulator is lo1 = lo + cl - 1000; lo1 + bl - 1000
        SUB k999
        SUB 00
        TAC c1n2
        STO lo           ; both carry: hi + ch + bh + 2
        CLA hi
        ADD ch
        ADD bh
        ADD 00
        ADD 00
        STO hi
        OUT hi
        OUT lo
        JMP loop
c1n2:   ADD k999         ; the first only: lo1 + bl, hi + ch + bh + 1
        ADD 00
        STO lo
        CLA hi
        ADD ch
        ADD bh
        ADD 00
        STO hi
        OUT hi
        OUT lo
        JMP loop
n1:     ADD bl           ; no first carry: lo1 = lo + cl, and lo1 + bl - 1000 is the accumulator plus bl
        TAC n1n2
        STO lo           ; the second only
        CLA hi
        ADD ch
        ADD bh
        ADD 00
        STO hi
        OUT hi
        OUT lo
        JMP loop
n1n2:   ADD k999         ; neither: lo + cl + bl
        ADD 00
        STO lo
        CLA hi
        ADD ch
        ADD bh
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
