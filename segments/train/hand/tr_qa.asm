; tr_qa by hand: products' first half, a b as (x + y)^2 / 4 - (x - y)^2 / 4 for the sizes x = |a|, y = |b|. Deck as
; tr_qa's: the count, then pairs a, b (cards). Punches the two sizes to look up in the table of quarter squares,
; each as its thousands (0 or 1) and the rest: the sum first and |x - y| second when a and b have the same sign
; (0 counts as positive), the other way round when they differ.
; Against the compiled tr_qa (about 40 instructions a pair): no sign flags kept and compared, the four sign cases
; going straight to the two orders; a size negated only when it is negative, in its own cell; the sum's carry by
; storing it first.
start:  INP n
pair:   CLA n
        SUB 00
        TAC done
        STO n
        INP x
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
x:      DAT 0
y:      DAT 0
t:      DAT 0
k0:     DAT 0
k999:   DAT 999
