; tr_walka by hand: walks by slopes. Deck as tr_walka's: the count; then each walk: n (a long), the start's value
; (a long), the slope's whole part (a long) and its thousandths, f0. Punches each of the n places' values, a long:
; the k-th is the start + fl(f0 + a k, 1000).
; Against the compiled tr_walka (about 35 instructions a place): nothing worked out again each place (it does
; 999 - w_lo and 999 - frac every time), the fraction first so its carry is one more on the low's add (v + w + 1
; carries at most once), the high once in the accumulator, and each low stored before its carry test.
start:  INP n
walk:   CLA n
        SUB 00
        TAC done
        STO n
        INP chi
        INP clo
        INP vhi
        INP vlo
        INP whi
        INP wlo
        INP fr
        INP f
place:  CLA clo          ; one place fewer to go
        SUB 00
        TAC borrow
        STO clo
body:   OUT vhi
        OUT vlo
        CLA f            ; the fraction: f + fr, and whether it carries
        ADD fr
        STO f
        SUB k999
        SUB 00
        TAC fnc
        STO f            ; it carries: f + fr - 1000, and the low gets w_lo + 1
        CLA vlo
        ADD wlo
        ADD 00
        JMP vtest
fnc:    CLA vlo          ; no carry: the low gets w_lo
        ADD wlo
vtest:  STO vlo
        SUB k999
        SUB 00
        TAC nc
        STO vlo          ; the low carries: hi + w_hi + 1
        CLA vhi
        ADD whi
        ADD 00
        STO vhi
        JMP place
nc:     CLA vhi
        ADD whi
        STO vhi
        JMP place
borrow: CLA chi          ; the low count was 0: a thousand from the high, or the walk is done
        SUB 00
        TAC walk
        STO chi
        CLA k999
        STO clo
        JMP body
done:   HRS 0
n:      DAT 0
chi:    DAT 0
clo:    DAT 0
vhi:    DAT 0
vlo:    DAT 0
whi:    DAT 0
wlo:    DAT 0
fr:     DAT 0
f:      DAT 0
k999:   DAT 999
