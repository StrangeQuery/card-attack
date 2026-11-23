; tr_mixa by hand: mixes' first half, a + (b - a) t / 100 needs (b - a) t, by quarter squares as tr_qa. Deck as
; tr_mixa's: the count, then a, b, t (cards; b - a a card). Punches the two sizes to look up, as tr_qa: this is
; train/hand/tr_qa.asm with b - a worked out first.
start:  INP n
pair:   CLA n
        SUB 00
        TAC done
        STO n
        INP a            ; x = b - a (a card)
        INP x
        CLA x
        SUB a
        STO x
        INP y
        CLA x
        TAC an
        CLA y
        TAC bp           ; a >= 0, b < 0: they differ
s:      CLA x            ; the same sign: the sum first
        ADD y
        STO t
        SUB k999
        SUB 00
        TAC s0
        STO t
        OUT 00
        OUT t
        JMP sd
s0:     OUT k0
        OUT t
sd:     CLA x            ; then |x - y|
        SUB y
        TAC sn
        STO t
        OUT k0
        OUT t
        JMP pair
sn:     CLA y
        SUB x
        STO t
        OUT k0
        OUT t
        JMP pair
bp:     CLA k0           ; y = -b
        SUB y
        STO y
d:      CLA x            ; they differ: |x - y| first
        SUB y
        TAC dn
        STO t
        JMP ds
dn:     CLA y
        SUB x
        STO t
ds:     OUT k0
        OUT t
        CLA x            ; then the sum
        ADD y
        STO t
        SUB k999
        SUB 00
        TAC d0
        STO t
        OUT 00
        OUT t
        JMP pair
d0:     OUT k0
        OUT t
        JMP pair
an:     CLA k0           ; x = -a
        SUB x
        STO x
        CLA y
        TAC bn
        JMP d            ; a < 0, b >= 0: they differ
bn:     CLA k0           ; both negative: the same sign
        SUB y
        STO y
        JMP s
done:   HRS 0
n:      DAT 0
a:      DAT 0
x:      DAT 0
y:      DAT 0
t:      DAT 0
k0:     DAT 0
k999:   DAT 999
