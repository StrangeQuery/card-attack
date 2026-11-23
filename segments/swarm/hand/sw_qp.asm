; sw_qp by hand: products from two quarter squares (longs), the difference placed as three cards (millions,
; thousands, rest). Deck as sw_qp's: the count; then each: its place (0, 1 or 2) and the two quarter squares.
; Against the compiled sw_qp: the difference once, then the place by a computed jump (JMP to a table, plus the
; place) straight to its punches.
start:  INP n
item:   CLA n
        SUB 00
        TAC done
        STO n
        INP pl
        INP th
        INP re
        INP bh
        INP bl
        CLA re           ; the difference, once
        SUB bl
        TAC b
        STO re
        CLA th
        SUB bh
        STO th
        JMP disp
b:      ADD k999
        ADD 00
        STO re
        CLA th
        SUB bh
        SUB 00
        STO th
disp:   CLA jt
        ADD pl
        STO jx
jx:     DAT 0            ; (JMP to the place's entry)
tbl:    JMP p0
        JMP p1
        JMP p2
p0:     CLA th           ; place 0: sign card, thousands, rest
        TAC m0
        OUT k0
        OUT th
        OUT re
        JMP item
m0:     ADD k999
        ADD 00
        STO th
        OUT km1
        OUT th
        OUT re
        JMP item
p1:     OUT th           ; place 1: times 1000
        OUT re
        OUT k0
        JMP item
p2:     CLA th           ; place 2: times a million: the rest, less 1000 if the whole is below 0
        TAC m2
        OUT re
        OUT k0
        OUT k0
        JMP item
m2:     CLA re
        SUB k999
        SUB 00
        STO re
        OUT re
        OUT k0
        OUT k0
        JMP item
done:   HRS 0
jt:     JMP tbl
n:      DAT 0
pl:     DAT 0
th:     DAT 0
re:     DAT 0
bh:     DAT 0
bl:     DAT 0
k0:     DAT 0
km1:    DAT -1
k999:   DAT 999
