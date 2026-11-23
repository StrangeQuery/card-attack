; sw_divd by hand: long division, a number's size by a long, a digit at a time. Deck as sw_divd's: the count; then
; each division: the long (thousands, rest; 1 to 99999), how many cards the size has, and those cards, the largest
; first. Punches each digit of the quotient, the largest first, then the remainder as a long.
; Against the compiled sw_divd: each digit by a fixed shift (SFT 13, 23, 33) rather than a shift instruction built
; each time; one routine for a digit's work, called with the digit in the accumulator (it keeps cell 99's way back,
; since every JMP writes 99); the remainder's shift as SFT 13 and SFT 21; the long taken away in place, put back
; once when it no longer fits, rather than worked out beside the remainder and copied over every time it does.
start:  INP n
div:    CLA n
        SUB 00
        TAC done
        STO n
        INP dh
        INP dl
        INP c
        CLA k0
        STO rh
        STO rl
part:   CLA c
        SUB 00
        TAC rem
        STO c
        INP p
        CLA p            ; its three digits, the largest first
        SFT 13
        JMP fit
        CLA p
        SFT 23
        JMP fit
        CLA p
        SFT 33
        JMP fit
        JMP part
rem:    OUT rh
        OUT rl
        JMP div
fit:    STO d            ; the remainder times ten plus d
        CLA 99
        STO ret
        CLA rl
        SFT 13
        STO t
        CLA rh
        SFT 10
        ADD t
        STO rh
        CLA rl
        SFT 21
        ADD d
        STO rl
        CLA k0
        STO f
fl:     CLA rl           ; the long taken away while it fits
        SUB dl
        TAC bw
        STO rl
        CLA rh
        SUB dh
        TAC r1
        STO rh
        CLA f
        ADD 00
        STO f
        JMP fl
bw:     ADD k999         ; the rest borrowed
        ADD 00
        STO rl
        CLA rh
        SUB dh
        SUB 00
        TAC r2
        STO rh
        CLA f
        ADD 00
        STO f
        JMP fl
r1:     CLA rl           ; it didn't fit: the rest put back
        ADD dl
        STO rl
        OUT f
        JMP ret
r2:     CLA rl           ; (borrowed: put back less the thousand)
        ADD dl
        SUB k999
        SUB 00
        STO rl
        OUT f
        JMP ret
done:   HRS 0
n:      DAT 0
dh:     DAT 0
dl:     DAT 0
c:      DAT 0
p:      DAT 0
d:      DAT 0
t:      DAT 0
rh:     DAT 0
rl:     DAT 0
f:      DAT 0
ret:    DAT 0
k0:     DAT 0
k999:   DAT 999
