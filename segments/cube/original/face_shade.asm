; Face shader. Deck: for each of the 6 faces, three consecutive corners a, b, c as projected (x, y). The cross
; product (b - a) x (c - b) is the face's projected area with a sign: negative faces the viewer, otherwise the
; face is hidden. A face's projected area is how squarely it faces us, so it is also the light, with the lamp
; at the eye. Punches 0 for a hidden face, else its brightness 8 + area/72 (at most 16).
start:  CLA kn6
        STO fc
fl:     INP ax
        INP ay
        INP bx
        INP by
        INP cx
        INP cy
        CLA bx           ; (bx - ax)(cy - by)
        SUB ax
        STO m1
        CLA cy
        SUB by
        STO m2
        JMP mul
        CLA prod
        STO cr
        CLA by           ; - (by - ay)(cx - bx)
        SUB ay
        STO m1
        CLA cx
        SUB bx
        STO m2
        JMP mul
        CLA cr
        SUB prod
        TAC vis
        OUT k0           ; hidden
        JMP fn
vis:    STO cr           ; area = -cross
        CLA k0
        SUB cr
        STO cr
        CLA k8
        STO v
sl:     CLA cr           ; v = 8 + area/72, at most 16
        SUB k72
        TAC sd
        STO cr
        CLA v
        ADD 00
        STO v
        SUB k16
        TAC sl
sd:     OUT v
fn:     CLA fc
        ADD 00
        STO fc
        TAC fl
        HRS 0
mul:    CLA 99           ; prod = m1 m2, by adding; returns through mret
        STO mret
        CLA k0
        STO prod
        CLA m2
        TAC mneg
ml:     CLA m2
        SUB 00
        TAC mret
        STO m2
        CLA prod
        ADD m1
        STO prod
        JMP ml
mneg:   CLA k0
        SUB m2
        STO m2
        CLA k0
        SUB m1
        STO m1
        JMP ml
mret:   DAT 0
ax:     DAT 0
ay:     DAT 0
bx:     DAT 0
by:     DAT 0
cx:     DAT 0
cy:     DAT 0
m1:     DAT 0
m2:     DAT 0
prod:   DAT 0
cr:     DAT 0
v:      DAT 0
fc:     DAT 0
k0:     DAT 0
k8:     DAT 8
k16:    DAT 16
k72:    DAT 72
kn6:    DAT -6
