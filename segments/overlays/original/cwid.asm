; Caption width machine, once per frame a caption shows. Deck: how many characters the caption has, n. Its box runs
; from the screen's left edge to x1 = 4 n + 4: a character's width of margin, then the text and its last gap.
start:  INP n
        CLA n
        ADD n
        ADD n
        ADD n
        ADD k4
        STO x
        OUT x
        HRS 0
n:      DAT 0
x:      DAT 0
k4:     DAT 4
