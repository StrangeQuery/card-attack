; tr_ldiff by hand: differences of longs, a - b, held to lo..hi (the thousands' bounds; the rest 0 at the low bound,
; 999 at the high). Deck as tr_ldiff's: the count, lo, hi, then pairs a, b. Punches each difference as a long.
; Against the compiled tr_ldiff: the high once, in the accumulator, borrow and all; each bound's punches straight
; from the constants, no cells set.
start:  INP n
        INP lo
        INP hi
pair:   CLA n
        SUB 00
        TAC done
        STO n
        INP ah
        INP al
        INP bh
        INP bl
        CLA al
        SUB bl
        TAC bw
        STO dl
        CLA ah
        SUB bh
        JMP cl
bw:     ADD k999
        ADD 00
        STO dl
        CLA ah
        SUB bh
        SUB 00
cl:     STO dh
        SUB lo
        TAC under
        CLA hi
        SUB dh
        TAC over
        OUT dh
        OUT dl
        JMP pair
under:  CLA hi           ; held to lo (then to hi, if hi is below lo, as tr_ldiff does)
        SUB lo
        TAC over
        OUT lo
        OUT k0
        JMP pair
over:   OUT hi
        OUT k999
        JMP pair
done:   HRS 0
n:      DAT 0
lo:     DAT 0
hi:     DAT 0
ah:     DAT 0
al:     DAT 0
bh:     DAT 0
bl:     DAT 0
dh:     DAT 0
dl:     DAT 0
k0:     DAT 0
k999:   DAT 999
