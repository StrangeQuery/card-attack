; tr_up4 by hand: glow made four times as fine. Deck as tr_up4's: the count of lines, then each line: its count of
; values, the values (cards 0..999). Between each value a and the next b, punches a, a + q, a + h, a + h + q, h half
; of b - a and q half of h, each cut toward zero.
; Halving by hand: five times it, then SFT 01 (one digit right, which cuts toward zero, as Rust's / does): 5 d fits
; the accumulator's four digits. The steps counted down once a line; a + q from the q in the accumulator, a + h + q
; from that, a + h from it.
start:  INP n
line:   CLA n
        SUB 00
        TAC done
        STO n
        INP m
        INP a
        CLA m            ; m - 1 steps
        SUB 00
        STO m
step:   CLA m
        SUB 00
        TAC line
        STO m
        INP b
        CLA b
        SUB a
        STO d
        ADD d
        ADD d
        ADD d
        ADD d
        SFT 01
        STO h
        ADD h
        ADD h
        ADD h
        ADD h
        SFT 01
        STO q
        OUT a
        ADD a            ; a + q (q is in the accumulator)
        STO t
        OUT t
        ADD h            ; a + h + q, kept; a + h from it
        STO u
        SUB q
        STO t
        OUT t
        OUT u
        CLA b
        STO a
        JMP step
done:   HRS 0
n:      DAT 0
m:      DAT 0
a:      DAT 0
b:      DAT 0
d:      DAT 0
h:      DAT 0
q:      DAT 0
t:      DAT 0
u:      DAT 0
