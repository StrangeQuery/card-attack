; Bounce machine, once per ball pixel. Deck: the bounced ray's x and y (out of 98), the pixel's b, the ball's
; height. A ray up and to the left catches the light (13); any other upward ray sees sky, high (8) or low (9). A
; downward ray falls from height (ball height + b) and meets the floor after t = 10 (height) / -y tenths of its
; length; past half a length the floor is lost in haze (12). Punches the colour, or -1 and t for the hit machine.
start:  INP rx
        INP ry
        INP b
        INP by
        CLA rx           ; highlight: rx <= -35 and ry >= 45
        ADD k35
        SUB 00
        TAC hl
        JMP nh
hl:     CLA ry
        SUB k45
        TAC nh
        OUT k13
        OUT k0
        HRS 0
nh:     CLA ry
        TAC dn
        SUB k51
        TAC lo
        OUT k8
        OUT k0
        HRS 0
lo:     OUT k9
        OUT k0
        HRS 0
dn:     STO m            ; m = -ry
        CLA k0
        SUB m
        STO m
        CLA by           ; t = 10 (height + b) / m
        ADD b
        SFT 10
        STO n
        CLA k0
        STO t
tl:     CLA n
        SUB m
        TAC td
        STO n
        CLA t
        ADD 00
        STO t
        JMP tl
td:     CLA t
        SUB k5
        TAC hit
        OUT k12
        OUT k0
        HRS 0
hit:    OUT kn1
        OUT t
        HRS 0
rx:     DAT 0
ry:     DAT 0
b:      DAT 0
by:     DAT 0
m:      DAT 0
n:      DAT 0
t:      DAT 0
k0:     DAT 0
kn1:    DAT -1
k5:     DAT 5
k8:     DAT 8
k9:     DAT 9
k12:    DAT 12
k13:    DAT 13
k35:    DAT 35
k45:    DAT 45
k51:    DAT 51
