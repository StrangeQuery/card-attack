; sw_qis by hand: quarter squares for signed products. Deck as sw_qis's: the count, then each: a, b (sizes, 0..999)
; and three signs (0 or 1). Punches the two sizes to look up, each as thousands (0 or 1) and the rest: a + b first and
; |a - b| second, or the other way round when an odd number of the signs is set.
; Against the compiled sw_qis: the parity by a computed jump (JMP to a table, plus the three signs' sum, stored in
; the cell it then runs), and the four orders as four tails, the sum's thousands in the tail rather than a cell.
start:  INP n
item:   CLA n
        SUB 00
        TAC done
        STO n
        INP a
        INP b
        INP p
        INP q
        INP r
        CLA a            ; |a - b|
        SUB b
        TAC sn
        STO sm
        JMP sum
sn:     CLA b
        SUB a
        STO sm
sum:    CLA a            ; a + b, and whether it passes 999
        ADD b
        STO bl
        SUB k999
        SUB 00
        TAC nc
        STO bl
        CLA jc           ; passed: the table of the tails with a thousand
        JMP par
nc:     CLA jn
par:    ADD p            ; the table's entry: the signs' sum, 0 to 3
        ADD q
        ADD r
        STO jx
jx:     DAT 0            ; (JMP to the entry)
tc:     JMP ce           ; with a thousand: even, odd, even, odd
        JMP co
        JMP ce
        JMP co
tn:     JMP ne           ; without
        JMP no
        JMP ne
        JMP no
ce:     OUT 00
        OUT bl
        OUT k0
        OUT sm
        JMP item
co:     OUT k0
        OUT sm
        OUT 00
        OUT bl
        JMP item
ne:     OUT k0
        OUT bl
        OUT k0
        OUT sm
        JMP item
no:     OUT k0
        OUT sm
        OUT k0
        OUT bl
        JMP item
done:   HRS 0
jc:     JMP tc
jn:     JMP tn
n:      DAT 0
a:      DAT 0
b:      DAT 0
p:      DAT 0
q:      DAT 0
r:      DAT 0
sm:     DAT 0
bl:     DAT 0
k0:     DAT 0
k999:   DAT 999
