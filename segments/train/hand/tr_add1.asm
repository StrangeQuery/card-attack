; Adding longs by hand, one long and the constant (k = 1). Deck (the three's): the count, k, the sign, the constant's
; two cards (hi, lo); then each long (hi, lo). Punches each sum as a long (hi, lo), lo 0..999. The compiled machine
; for all three cases (tr_add, since removed) took 26-28 instructions an item here; this one 18-19: 999 - cl once a
; run (not once an item), k's test out of the loop (a machine per case), and the carry's +1 folded into the sum of the
; highs.
start:  INP n
        INP k
        INP s2
        INP ch
        INP m
        CLA m            ; m = cl - 999: lo + m is lo + cl - 999, over 0 when the lows carry
        SUB k999
        STO m
loop:   CLA n
        SUB 00
        TAC done
        STO n
        INP hi
        INP lo
        CLA lo
        ADD m
        SUB 00
        TAC nc           ; lo + cl - 1000 below 0: no carry
        STO lo           ; the carry: lo + cl - 1000, and hi + ch + 1
        CLA hi
        ADD ch
        ADD 00
        STO hi
        OUT hi
        OUT lo
        JMP loop
nc:     ADD k999         ; no carry: lo + cl, and hi + ch
        ADD 00
        STO lo
        CLA hi
        ADD ch
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
k999:   DAT 999
