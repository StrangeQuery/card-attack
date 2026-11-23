; tr_addc by hand: sums of cards, each term added or taken away, and a constant. Deck as tr_addc's: the count, the
; constant, how many terms (0 to 6), each one's sign (1 or -1); then each sum's terms. Punches each sum (a card).
; Against the compiled tr_addc (which walks an array of the signs for every sum): the program writes itself once a
; run, from the signs: each term's slot gets ADD t or SUB t, and the slot after the last a jump to the end, so a sum
; is straight-line code, CLA c, then INP t and ADD t (or SUB t) a term, STO s and OUT s. The sum stays in the
; accumulator until it's done.
start:  INP n
        INP c
        INP k
patch:  CLA k            ; a slot for each sign
        SUB 00
        TAC endp
        STO k
        INP sg
        CLA sg
        TAC neg
        CLA kadd
        JMP put
neg:    CLA ksub
put:    STO s1o          ; (this STO's address walks along the slots: 2 a slot)
        CLA put
        ADD k2
        STO put
        JMP patch
endp:   CLA put          ; the slot after the last: its INP cell becomes JMP fin
        SUB 00
        STO pj
        CLA kjmp
pj:     STO s1i          ; (its address set just above)
sum:    CLA n
        SUB 00
        TAC done
        STO n
        CLA c
s1i:    INP t
s1o:    DAT 0
s2i:    INP t
s2o:    DAT 0
s3i:    INP t
s3o:    DAT 0
s4i:    INP t
s4o:    DAT 0
s5i:    INP t
s5o:    DAT 0
s6i:    INP t
s6o:    DAT 0
s7i:    JMP fin          ; (after six terms)
fin:    STO s
        OUT s
        JMP sum
done:   HRS 0
kadd:   ADD t
ksub:   SUB t
kjmp:   JMP fin
k2:     DAT 2
n:      DAT 0
c:      DAT 0
k:      DAT 0
sg:     DAT 0
t:      DAT 0
s:      DAT 0
