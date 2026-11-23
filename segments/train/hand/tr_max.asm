; tr_max by hand: the greater of two longs (thousands, then the rest 0..999). Deck as tr_max's: the count, then pairs a,
; b (longs). Punches each.
; Against the compiled one (17 instructions a pair, already tight but for a spare STO): two pairs a loop, so the
; count is tested once for two, the second pair's "a below b" falling through to the loop; an odd last pair by
; itself. Made from one template with tr_max, tr_min and tr_lt.
start:  INP n
two:    CLA n
        SUB k2
        TAC tail
        STO n
p:      INP ah
        INP al
        INP bh
        INP bl
        CLA ah
        SUB bh
        TAC pb   ; thousands below: a below b
        SUB 00
        TAC pe   ; equal: the rests decide
pa:     OUT ah
        OUT al
        JMP q
pe:     CLA al
        SUB bl
        TAC pb
        JMP pa
pb:     OUT bh
        OUT bl
q:      INP ah
        INP al
        INP bh
        INP bl
        CLA ah
        SUB bh
        TAC qb   ; thousands below: a below b
        SUB 00
        TAC qe   ; equal: the rests decide
qa:     OUT ah
        OUT al
        JMP two
qe:     CLA al
        SUB bl
        TAC qb
        JMP qa
qb:     OUT bh
        OUT bl
        JMP two
tail:   ADD 00           ; n - 2 + 1: one pair left, or none
        TAC done
r:      INP ah
        INP al
        INP bh
        INP bl
        CLA ah
        SUB bh
        TAC rb   ; thousands below: a below b
        SUB 00
        TAC re   ; equal: the rests decide
ra:     OUT ah
        OUT al
        JMP done
re:     CLA al
        SUB bl
        TAC rb
        JMP ra
rb:     OUT bh
        OUT bl
done:   HRS 0
n:      DAT 0
ah:     DAT 0
al:     DAT 0
bh:     DAT 0
bl:     DAT 0
k0:     DAT 0
k2:     DAT 2
