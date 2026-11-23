; tr_qb by hand: products' second half, the difference of two quarter squares (longs), as a long, or over 1000
; rounded down (s 3). Deck as tr_qb's: the count, s (0 or 3), then pairs of longs. Punches each as a long.
; Against the compiled tr_qb: s tested once a run, not once a pair (a loop for each), and the over-1000 loop never
; keeps the low card at all.
start:  INP n
        INP s
        CLA s
        SUB 00
        TAC l0
l3:     CLA n            ; s 3: hi - bh, less 1 when the lows borrow, as a long
        SUB 00
        TAC done
        STO n
        INP hi
        INP lo
        INP bh
        INP bl
        CLA lo
        SUB bl
        TAC b3
        CLA hi
        SUB bh
        TAC n3
        STO t
        OUT k0
        OUT t
        JMP l3
b3:     CLA hi
        SUB bh
        SUB 00
        TAC n3
        STO t
        OUT k0
        OUT t
        JMP l3
n3:     ADD k999         ; below 0: -1 and the rest
        ADD 00
        STO t
        OUT km1
        OUT t
        JMP l3
l0:     CLA n            ; s 0: the difference as it is
        SUB 00
        TAC done
        STO n
        INP hi
        INP lo
        INP bh
        INP bl
        CLA lo
        SUB bl
        TAC b0
        STO lo
        CLA hi
        SUB bh
        STO hi
        OUT hi
        OUT lo
        JMP l0
b0:     ADD k999
        ADD 00
        STO lo
        CLA hi
        SUB bh
        SUB 00
        STO hi
        OUT hi
        OUT lo
        JMP l0
done:   HRS 0
n:      DAT 0
s:      DAT 0
hi:     DAT 0
lo:     DAT 0
bh:     DAT 0
bl:     DAT 0
t:      DAT 0
k0:     DAT 0
km1:    DAT -1
k999:   DAT 999
