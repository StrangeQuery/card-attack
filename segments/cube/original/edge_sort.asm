; Edge sorter. Deck: the 16 numbers the vertex machine punched (x, y of each corner), then 12 edges as corner pairs.
; Punches each edge as ytop, ybottom, xtop, xbottom.
start:  CLA stovt        ; read the 16 corner numbers into vt
        STO wv
        CLA kn16
        STO c
rv:     INP v
        CLA v
wv:     DAT 0            ; becomes STO vt+i
        CLA wv
        ADD 00
        STO wv
        CLA c
        ADD 00
        STO c
        TAC rv
        CLA kn12
        STO c
el:     INP a
        INP b
        CLA a            ; corner a: x at vt[2a], y at vt[2a+1]
        ADD a
        ADD clavt
        STO r1
r1:     DAT 0
        STO xa
        CLA r1
        ADD 00
        STO r2
r2:     DAT 0
        STO ya
        CLA b            ; corner b
        ADD b
        ADD clavt
        STO r3
r3:     DAT 0
        STO xb
        CLA r3
        ADD 00
        STO r4
r4:     DAT 0
        STO yb
        SUB ya           ; top first
        TAC sw
        OUT ya
        OUT yb
        OUT xa
        OUT xb
        JMP en
sw:     OUT yb
        OUT ya
        OUT xb
        OUT xa
en:     CLA c
        ADD 00
        STO c
        TAC el
        HRS 0
v:      DAT 0
c:      DAT 0
a:      DAT 0
b:      DAT 0
xa:     DAT 0
ya:     DAT 0
xb:     DAT 0
yb:     DAT 0
kn16:   DAT -16
kn12:   DAT -12
stovt:  DAT vt+600       ; STO vt
clavt:  DAT vt+100       ; CLA vt
vt:     DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
        DAT 0
