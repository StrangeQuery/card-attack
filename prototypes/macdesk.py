"""Prototype: the Mac opening. A faithful System 6 Finder on a Mac Classic's 512x342 one-bit screen: the
homework windows open, the pointer closes them one by one, a floppy goes in and mounts as Card Attack, its window
opens, the Card Attack application is double-clicked and launches (the zoom rects grow to the whole screen).

    python macdesk.py OUT.mp4          a preview, 2x nearest neighbour (1024x684), 12 fps

Fonts: Chicago 12, Geneva 9/12 from System 6 (macfonts/, joshua stein's classic-mac-fonts; see CREDITS.md).
"""
import copy
import os
import subprocess
import sys

import numpy as np

W, H, FPS = 512, 342, 12
HERE = os.path.dirname(os.path.abspath(__file__))

# ---- fonts ----------------------------------------------------------------------------------------------------------

def load_bdf(name):
    glyphs, cur = {}, None
    for line in open(os.path.join(HERE, "macfonts", name + ".bdf"), encoding="latin-1"):
        k = line.split()
        if not k:
            continue
        if k[0] == "ENCODING":
            cur = {"code": int(k[1]), "rows": []}
        elif k[0] == "DWIDTH":
            cur["adv"] = int(k[1])
        elif k[0] == "BBX":
            cur["bbx"] = tuple(map(int, k[1:5]))
        elif k[0] == "BITMAP":
            cur["bits"] = True
        elif k[0] == "ENDCHAR":
            w, h, _, _ = cur["bbx"]
            nb = max(1, (w + 7) // 8) * 8
            cur["img"] = np.array([[(int(r, 16) >> (len(r) * 4 - 1 - i)) & 1 for i in range(w)]
                                   for r in cur["rows"]], bool).reshape(h, w) if h else np.zeros((0, w), bool)
            glyphs[cur["code"]] = cur
            cur = None
        elif cur is not None and cur.get("bits"):
            cur["rows"].append(k[0])
    return glyphs

CHICAGO, GENEVA9, GENEVA12 = load_bdf("Chicago-12"), load_bdf("Geneva-9"), load_bdf("Geneva-12")
APPLE = chr(20)                                           # the Apple menu's logo in Chicago

def codes(s):
    """The fonts are in MacRoman (the Apple's 20 stays itself)."""
    return [ord(c) if ord(c) < 32 else b for c in s for b in (c.encode("mac_roman") if ord(c) >= 32 else [0])]

def width(s, font):
    return sum(font[c]["adv"] for c in codes(s) if c in font)

def text(img, x, base, s, font, ink=True):
    for c in codes(s):
        g = font.get(c)
        if g is None:
            continue
        w, h, xo, yo = g["bbx"]
        top = base - yo - h
        y0, y1, x0, x1 = max(0, top), min(H, top + h), max(0, x + xo), min(W, x + xo + w)
        if y1 > y0 and x1 > x0:
            sub = g["img"][y0 - top:y1 - top, x0 - x - xo:x1 - x - xo]
            img[y0:y1, x0:x1][sub] = ink
        x += g["adv"]
    return x

# ---- drawing --------------------------------------------------------------------------------------------------------

def fill(img, l, t, r, b, v):                            # inclusive rectangles, like QuickDraw's pixels
    img[max(0, t):min(H, b + 1), max(0, l):min(W, r + 1)] = v

def frame(img, l, t, r, b, v=True):
    fill(img, l, t, r, t, v); fill(img, l, b, r, b, v); fill(img, l, t, l, b, v); fill(img, r, t, r, b, v)

def invert_frame(img, l, t, r, b):
    """An XOR outline, as the Finder draws its zoom rects."""
    l, t, r, b = max(0, l), max(0, t), min(W - 1, r), min(H - 1, b)
    img[t, l:r + 1] ^= True
    img[b, l:r + 1] ^= True if b != t else False
    img[t + 1:b, l] ^= True
    img[t + 1:b, r] ^= True if r != l else False

def desktop():
    y, x = np.mgrid[0:H, 0:W]
    return ((x + y) % 2 == 0)                             # the 50% grey: every other pixel black

def screen_corners(img):
    for (cy, cx, sy, sx) in ((0, 0, 1, 1), (0, W - 1, 1, -1), (H - 1, 0, -1, 1), (H - 1, W - 1, -1, -1)):
        for row, n in enumerate((5, 3, 2, 1, 1)):        # the CRT's rounded corners, black
            img[cy + sy * row, cx: cx + sx * n: sx] = True if sx > 0 else True
            if sx < 0:
                img[cy + sy * row, cx - n + 1: cx + 1] = True

def menubar(img, titles, inverted=None):
    fill(img, 0, 0, W - 1, 18, False)
    fill(img, 0, 19, W - 1, 19, True)
    x = 10
    for i, s in enumerate(titles):
        if s == "CLOCK":                                  # the Alarm Clock flashing in the Apple's place
            small = np.array([[c == "X" for c in row] for row in SMALL_CLOCK])
            img[3:3 + small.shape[0], x:x + small.shape[1]] |= small
            x += 11 + 17
            continue
        w = width(s, CHICAGO)
        if inverted == i:
            fill(img, x - 6, 1, x + w + 5, 18, True)
        text(img, x, 14, s, CHICAGO, ink=inverted != i)
        x += w + (17 if i == 0 else 14)

# ---- icons: 32x32, ink and mask -------------------------------------------------------------------------------------

def blank():
    return np.zeros((32, 32), bool)

def icon_folder():
    ink, m = blank(), blank()
    fill_(m, 1, 6, 12, 9); fill_(m, 0, 9, 31, 27)
    out(ink, m)
    ink[10, 1:31] = True                                   # the tab's line across the body
    return ink, m

def icon_doc(lines=True):
    ink, m = blank(), blank()
    fill_(m, 4, 1, 26, 31)
    out(ink, m)
    for i in range(7):                                    # the dog-ear
        m[1 + i, 20 + i + 1:27] = False
        ink[1 + i, 20 + i + 1:27] = False
        ink[1 + i, 20 + i] = True
    ink[7, 20:27] = True
    ink[1:8, 20] = True
    if lines:
        for y in range(11, 28, 3):
            ink[y, 8:23 - (5 if y == 26 else 0)] = True
    return ink, m

def icon_harddisk():
    ink, m = blank(), blank()
    fill_(m, 0, 10, 31, 23)
    out(ink, m)
    ink[19, 1:31] = True
    ink[21, 4:7] = True                                   # the light
    return ink, m

def icon_floppy():
    ink, m = blank(), blank()
    fill_(m, 2, 2, 29, 29)
    out(ink, m)
    m[2, 29] = False; ink[2, 29] = False; ink[3, 28] = True   # the clipped corner
    frame_(ink, 9, 2, 22, 12)                             # the shutter
    ink[4:11, 18:21] = True                               # its window
    frame_(ink, 6, 16, 25, 29)                            # the label
    for y in (20, 23, 26):
        ink[y, 9:23] = True
    return ink, m

def icon_trash():
    ink, m = blank(), blank()
    fill_(m, 12, 1, 19, 4); fill_(m, 5, 4, 26, 8); fill_(m, 7, 8, 24, 31)
    out(ink, m)
    ink[8, 5:27] = True
    for x in (11, 15, 19):                                # the ridges
        ink[11:29, x] = True
        ink[11:29, x + 1] = True
    return ink, m

def icon_card():
    """Card Attack's application icon: a punched card, its corner clipped."""
    ink, m = blank(), blank()
    fill_(m, 0, 7, 31, 25)
    for i in range(5):
        m[7 + i, 0:5 - i] = False
    out(ink, m)
    for i in range(5):
        ink[7 + i, 5 - i] = True
    holes = [(3, 11), (6, 14), (9, 17), (12, 20), (15, 13), (18, 16), (21, 19), (24, 22), (27, 12), (9, 21),
             (21, 11), (15, 22), (6, 18), (24, 15)]
    for x, y in holes:
        ink[y:y + 2, x] = True
    return ink, m

def fill_(a, l, t, r, b):
    a[t:b + 1, l:r + 1] = True

def frame_(a, l, t, r, b):
    a[t, l:r + 1] = a[b, l:r + 1] = True
    a[t:b + 1, l] = a[t:b + 1, r] = True

def out(ink, m):
    """Outline the mask: a pixel is ink if it's in the mask and next to one outside it."""
    p = np.pad(m, 1)
    edge = m & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])
    ink |= edge

def icon_caution():
    ink, m = blank(), blank()
    for y in range(2, 30):                                # a triangle, point up
        half = (y - 2) * 15 // 27
        m[y, 15 - half:17 + half] = True
    out(ink, m)
    ink[9:22, 15:17] = True                               # the !
    ink[24:27, 15:17] = True
    return ink, m

def icon_clock():
    ink, m = blank(), blank()
    y, x = np.mgrid[0:32, 0:32]
    d = (x - 15.5) ** 2 + (y - 17.5) ** 2
    m |= d <= 13.5 ** 2
    fill_(m, 4, 1, 9, 6); fill_(m, 22, 1, 27, 6)          # the bells
    out(ink, m)
    ink |= (d <= 11.5 ** 2) & (d > 10.5 ** 2)
    ink[9:18, 15:17] = True                               # the hands: ten past
    for k in range(7):
        ink[17 - k // 2, 16 + k] = True
    fill_(ink, 4, 1, 9, 6); fill_(ink, 22, 1, 27, 6)
    ink[28:31, 7:9] = True; ink[28:31, 23:25] = True      # the feet
    m[28:31, 7:9] = True; m[28:31, 23:25] = True
    return ink, m

SMALL_CLOCK = [".XX.....XX.", "XX.XXXXX.XX", "..X..X..X..", ".X...X...X.", ".X...X...X.", ".X...XXX.X.",
               ".X.......X.", "..X.....X..", "...XXXXX...", "..X.....X..", ".X.......X."]

ICONS = {"folder": icon_folder(), "doc": icon_doc(), "hd": icon_harddisk(), "floppy": icon_floppy(),
         "trash": icon_trash(), "card": icon_card(),
         "caution": icon_caution(), "clock": icon_clock()}

def draw_icon(img, kind, x, y, name, selected=False, on_desktop=True):
    ink, m = ICONS[kind]
    sub = img[y:y + 32, x:x + 32]
    if selected == "open":
        sub[m] = False
        yy, xx = np.mgrid[y:y + 32, x:x + 32]
        sub[m & ((xx + yy) % 2 == 0)] = True
        sub[edge_of(m)] = True
        selected = True
    elif selected:
        sub[m] = True                                     # inverted: black through, the outline white
        sub[m & ink & np.pad(m, 1)[1:-1, 1:-1] & ~edge_of(m)] = False
        sub[edge_of(m)] = True
    else:
        sub[m] = False
        sub[ink] = True
    w = width(name, GENEVA9)
    nx, nb = x + 16 - w // 2, y + 32 + 10
    if on_desktop or selected:
        fill(img, nx - 1, nb - 9, nx + w, nb + 2, selected)
    text(img, nx, nb, name, GENEVA9, ink=not selected)

def edge_of(m):
    p = np.pad(m, 1)
    return m & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])

# ---- windows --------------------------------------------------------------------------------------------------------

def box(img, l, t):                                       # the close box, 11 pixels square in its white margin
    fill(img, l - 1, t - 1, l + 11, t + 11, False)
    frame(img, l, t, l + 10, t + 10)

def window(img, w, active):
    l, t, r, b = w["rect"]
    fill(img, l, t, r, b, False)
    frame(img, l, t, r, b)
    fill(img, l + 1, b + 1, r + 1, b + 1, True)           # the drop shadow
    fill(img, r + 1, t + 1, r + 1, b + 1, True)
    fill(img, l, t + 18, r, t + 18, True)                 # the title bar's line
    title = w["title"]
    tw = width(title, CHICAGO)
    tx = (l + r) // 2 - tw // 2
    if active:
        for y in range(t + 4, t + 15, 2):
            fill(img, l + 2, y, r - 2, y, True)
        fill(img, tx - 6, t + 3, tx + tw + 5, t + 15, False)
        box(img, l + 8, t + 4)
        if w.get("pressed"):                              # the close box held down: its starburst
            cx, cy = l + 13, t + 9
            for dx, dy in ((1, 0), (0, 1), (1, 1), (1, -1)):
                for k in range(-4, 5):
                    if abs(k) > 1:
                        img[cy + dy * k, cx + dx * k] = True
        if w.get("zoom", True):
            box(img, r - 18, t + 4)
            frame(img, r - 18, t + 4, r - 13, t + 9)
    text(img, tx, t + 14, title, CHICAGO)
    top = t + 19
    if w.get("info"):                                     # the Finder's header: items, in disk, available
        parts = w["info"]
        xs = [l + 10, (l + r) // 2 - width(parts[1], GENEVA9) // 2, r - 26 - width(parts[2], GENEVA9)]
        for s, x in zip(parts, xs):
            text(img, x, top + 10, s, GENEVA9)
        fill(img, l + 1, top + 13, r - 1, top + 13, True)
        fill(img, l + 1, top + 15, r - 1, top + 15, True)
        top += 16
    if w.get("scroll", True):
        fill(img, r - 15, t + 18, r - 15, b, True)        # the scroll bars and the grow box
        fill(img, l, b - 15, r, b - 15, True)
        if active:
            arrow(img, r - 14, t + 19, up=True)
            arrow(img, r - 14, b - 31, up=False)
            harrow(img, l + 1, b - 14, left=True)
            harrow(img, r - 31, b - 14, left=False)
            frame(img, r - 11, b - 11, r - 3, b - 3)
            frame(img, r - 13, b - 13, r - 7, b - 7)
            fill(img, r - 12, b - 12, r - 8, b - 8, False)
        cl, ct, cr, cb = l + 1, top, r - 16, b - 16
    else:
        cl, ct, cr, cb = l + 1, top, r - 1, b - 1
    if w["draw"] is book_report:
        typed = BOOK[:w["typed"]]
        lines = typed.split("\n")
        for i, line in enumerate(lines):
            text(img, cl + 10, ct + 18 + 16 * i, line, GENEVA12)
        if active and w.get("caret"):                     # the insertion point, blinking
            x = cl + 10 + width(lines[-1], GENEVA12)
            base = ct + 18 + 16 * (len(lines) - 1)
            fill(img, x, base - 11, x, base + 2, True)
    else:
        w["draw"](img, cl, ct, cr, cb, active)

ARROW_UP = ["....X....", "...X.X...", "..X...X..", ".X.....X.", "XXXX.XXXX", "...X.X...", "...X.X...",
            "...XXX..."]

def arrow(img, l, t, up):
    fill(img, l, t + (15 if up else 0), l + 14, t + (15 if up else 0), True)
    rows = ARROW_UP if up else ARROW_UP[::-1]
    for j, row in enumerate(rows):
        for i, c in enumerate(row):
            if c == "X":
                img[t + 4 + j, l + 3 + i] = True

def harrow(img, l, t, left):
    fill(img, l + (15 if left else 0), t, l + (15 if left else 0), t + 14, True)
    for j, row in enumerate(ARROW_UP if left else ARROW_UP[::-1]):
        for i, c in enumerate(row):
            if c == "X":
                img[t + 3 + i, l + 4 + j] = True

def lines_of(s, x, y0, step=16, font=GENEVA12):
    def draw(img, l, t, r, b, active):
        for i, line in enumerate(s.split("\n")):
            text(img, l + x, t + y0 + i * step, line, font)
    return draw

def icons_in(items):
    def draw(img, l, t, r, b, active):
        for kind, dx, dy, name in items:
            draw_icon(img, kind, l + dx, t + dy, name, on_desktop=False)
    return draw

def notepad(img, l, t, r, b, active):
    lines_of("do homework\nbuy floppies\ncall mom back\n\n\n\n\n", 8, 16)(img, l, t, r, b, active)
    fill(img, l, b - 14, l + 14, b - 14, True)            # the page's turned-up corner and its number
    for i in range(14):
        img[b - 14 + i, l + i] = True
    text(img, (l + r) // 2 - 3, b - 3, "1", GENEVA9)

def calculator(img, l, t, r, b, active):
    fill(img, l + 8, t + 6, r - 8, t + 22, False)
    frame(img, l + 8, t + 6, r - 8, t + 22)
    s = "58008"
    text(img, r - 12 - width(s, CHICAGO), t + 19, s, CHICAGO)
    keys = ["CE=*", "789/", "456-", "123+", "0 . "]
    for j, row in enumerate(keys):
        for i, c in enumerate(row):
            if c == " ":
                continue
            kl, kt = l + 8 + i * 20, t + 30 + j * 17
            kr = kl + 16 + (20 if (c == "0") else 0)
            frame(img, kl, kt, kr, kt + 13)
            fill(img, kl + 1, kt + 14, kr + 1, kt + 14, True)
            fill(img, kr + 1, kt + 1, kr + 1, kt + 14, True)
            text(img, (kl + kr) // 2 - width(c, CHICAGO) // 2 + 1, kt + 11, c, CHICAGO)
    keys = None

ALERT = (96, 96, 416, 196)
ICLOUD_ALERT = {"icon": "caution", "lines": ["iCloud: You are using 589K of 640K.", "You can't have more."]}
REMINDER = {"icon": "clock", "lines": ["Demos tonight in Derek's room.", "Did you check out that new one?"]}
OK_BUTTON = (330, 162, 389, 181)

def rounded(img, l, t, r, b, v=True, fill_with=None):
    if fill_with is not None:
        fill(img, l + 1, t + 1, r - 1, b - 1, fill_with)
    fill(img, l + 3, t, r - 3, t, v); fill(img, l + 3, b, r - 3, b, v)
    fill(img, l, t + 3, l, b - 3, v); fill(img, r, t + 3, r, b - 3, v)
    for (x, y, sx, sy) in ((l, t, 1, 1), (r, t, -1, 1), (l, b, 1, -1), (r, b, -1, -1)):
        img[y + sy, x + 2 * sx] = v; img[y + 2 * sy, x + sx] = v
        img[y + sy, x + sx] = False if fill_with is None else fill_with

def alert(img, spec):
    """A System 6 alert: the double border, the caution icon, Chicago, the default button ringed."""
    l, t, r, b = ALERT
    fill(img, l, t, r, b, True)
    fill(img, l + 1, t + 1, r - 1, b - 1, False)
    fill(img, l + 3, t + 3, r - 3, b - 3, True)
    fill(img, l + 5, t + 5, r - 5, b - 5, False)
    pressed = spec.get("pressed", False)
    draw_icon(img, spec["icon"], l + 20, t + 18, "", on_desktop=False)
    for i, line in enumerate(spec["lines"]):
        text(img, l + 72, t + 30 + 16 * i + (8 if len(spec["lines"]) == 1 else 0), line, CHICAGO)
    bl, bt, br, bb = OK_BUTTON
    for k in range(3):                                    # the default button's thick ring
        rounded(img, bl - 4 + k, bt - 4 + k, br + 4 - k, bb + 4 - k)
    rounded(img, bl, bt, br, bb, fill_with=pressed)
    text(img, (bl + br) // 2 - width("OK", CHICAGO) // 2, bb - 5, "OK", CHICAGO, ink=not pressed)

BOOK = ("Book Report: Moby Dick\nby me\n\nIt is about a whale. The whale is\nvery big. I did not finish it "
        "but\nI think the whale wins.\n\nTHE E")
BOOK_START = BOOK.index("I think")

def book_report(img, l, t, r, b, active, w=None):
    pass

# the disk the demo was written on: a Lightspeed C project (its π), the sources, the assembly, the build
FLOPPY = [("doc", 18, 12, "CardAttack.\u03c0"), ("doc", 104, 12, "CardAttack.\u03c0.rsrc"), ("doc", 190, 12, "main.c"),
          ("doc", 276, 12, "tunnel.c"), ("doc", 18, 74, "dither.c"), ("doc", 104, 74, "CARDIAC.h"),
          ("doc", 190, 74, "blit.a"), ("card", 276, 74, "Card Attack")]
FLOPPY_APP = (276, 74)

# the desk, back to front
def homework_windows():
    return [
        {"title": "Homework", "rect": (14, 32, 318, 214), "info": ("5 items", "382K in disk", "18K available"),
         "draw": icons_in([("doc", 20, 14, "Book Report"), ("doc", 92, 14, "Math"), ("folder", 164, 14, "Science Fair"),
                           ("doc", 236, 14, "Excuses"), ("doc", 20, 74, "Excuses 2")]),
         "folder_at": (464, 28)},
        {"title": "iCloud", "rect": (196, 132, 446, 262), "info": ("1 item", "589K in disk", "51K available"),
         "draw": icons_in([("doc", 100, 16, "Untitled")]), "folder_at": (464, 28)},
        {"title": "iTunes", "rect": (150, 36, 420, 156), "info": ("3 items", "1,104K in disk", "18K available"),
         "draw": icons_in([("folder", 26, 14, "Rick Astley"), ("folder", 110, 14, "Cyndi Lauper"),
                           ("folder", 194, 14, "Bonnie Tyler")]), "folder_at": (464, 28)},
        {"title": "Math", "rect": (196, 112, 470, 300),
         "draw": lines_of("1.  7 x 8 = 54\n2.  144 / 12 = a lot\n3.  Show your work:  no\n4.  x = 7 (I asked a "
                          "friend)", 10, 18)},
        {"title": "Note Pad", "rect": (30, 176, 196, 318), "scroll": False, "zoom": False, "draw": notepad},
        {"title": "Calculator", "rect": (330, 40, 427, 190), "scroll": False, "zoom": False, "draw": calculator},
        {"title": "Book Report", "rect": (56, 62, 398, 262), "draw": book_report, "typed": BOOK_START, "caret": True},
    ]

# ---- the pointer ----------------------------------------------------------------------------------------------------

ARROW = ([0x0000, 0x4000, 0x6000, 0x7000, 0x7800, 0x7C00, 0x7E00, 0x7F00, 0x7F80, 0x7C00, 0x6C00, 0x4600, 0x0600,
          0x0300, 0x0300, 0x0000],
         [0xC000, 0xE000, 0xF000, 0xF800, 0xFC00, 0xFE00, 0xFF00, 0xFF80, 0xFFC0, 0xFFE0, 0xFE00, 0xEF00, 0xCF00,
          0x8780, 0x0780, 0x0380], (1, 1))
WATCH = ([0x3F00, 0x3F00, 0x3F00, 0x3F00, 0x4080, 0x8440, 0x8440, 0x8460, 0x9C40, 0x8040, 0x4080, 0x3F00, 0x3F00,
          0x3F00, 0x3F00, 0x0000],
         [0x3F00, 0x3F00, 0x3F00, 0x3F00, 0x7F80, 0xFFC0, 0xFFC0, 0xFFE0, 0xFFC0, 0xFFC0, 0x7F80, 0x3F00, 0x3F00,
          0x3F00, 0x3F00, 0x0000], (8, 8))

def watch(step):
    """The watch cursor with its hand at one of eight places, as it turned while the Mac was busy."""
    mask = list(WATCH[1])
    m = np.array([[(row >> (15 - i)) & 1 for i in range(16)] for row in mask], bool)
    p = np.pad(m, 1)
    inner = m & p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
    face = np.zeros_like(m)
    face[5:11, 1:10] = True
    ink = m & ~(inner & face)
    ink[4:12, 0] = ink[4:12, 10] = False
    ink[5:11, 0] = ink[5:11, 10] = True
    ink[7, 11] = True                                     # the crown
    dx, dy = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)][step % 8]
    for k in range(3):
        ink[8 + dy * k, 5 + dx * k] = True
    data = [int("".join("1" if v else "0" for v in row), 2) for row in ink]
    return data, mask, (6, 8)

IBEAM = ([0x0C60, 0x0280, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100,
          0x0280, 0x0C60, 0x0000], [0x0C60, 0x0280, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100, 0x0100,
          0x0100, 0x0100, 0x0100, 0x0280, 0x0C60, 0x0000], (7, 8))   # the text editor's I-beam

def cursor(img, x, y, kind):
    if kind.startswith("watch"):
        data, mask, (hx, hy) = watch(int(kind[5:] or 0))
    elif kind == "ibeam":
        data, mask, (hx, hy) = IBEAM
    else:
        data, mask, (hx, hy) = ARROW
    for j in range(16):
        for i in range(16):
            yy, xx = y - hy + j, x - hx + i
            if 0 <= yy < H and 0 <= xx < W and (mask[j] >> (15 - i)) & 1:
                img[yy, xx] = bool((data[j] >> (15 - i)) & 1)

# ---- the tunnel, in one bit ----------------------------------------------------------------------------------------

def atkinson(grey):
    """Bill Atkinson's dither from MacPaint: each pixel's error, an eighth each to six neighbours (3/4 of it kept)."""
    h, w = grey.shape
    g = grey.astype(np.float32).copy()
    out = np.zeros((h, w), bool)
    for y in range(h):
        row, nxt, nxt2 = g[y], g[y + 1] if y + 1 < h else None, g[y + 2] if y + 2 < h else None
        for x in range(w):
            old = row[x]
            black = old < 0.5
            out[y, x] = black
            e = (old - (0.0 if black else 1.0)) / 8
            if x + 1 < w: row[x + 1] += e
            if x + 2 < w: row[x + 2] += e
            if nxt is not None:
                if x > 0: nxt[x - 1] += e
                nxt[x] += e
                if x + 1 < w: nxt[x + 1] += e
            if nxt2 is not None:
                nxt2[x] += e
    return out

CHUNK = 4                                                 # a Mac Plus's honest effort: 128x86 in 4x4 blocks
TH, TW = -(-H // CHUNK), W // CHUNK
YY, XX = (np.mgrid[0:TH, 0:TW].astype(np.float32) + 0.5) * CHUNK
DX, DY = XX - W / 2 + 0.5, YY - H / 2 + 0.5
RADIUS = np.hypot(DX, DY) + 1e-3
ANGLE = np.arctan2(DY, DX) / (2 * np.pi)

def tunnel(t):
    """The film's tunnel: a checkerboard down a round tunnel, twisting as you fly, the far end into black. Cream
    is light and red is mid-grey; the dither makes the greys."""
    depth = 2400 / RADIUS
    u = ANGLE * 16 + 0.6 * np.sin(t * 0.9) + depth * 0.012 * np.sin(t * 0.6)
    v = depth / 3.5 + t * 2.2
    check = (np.floor(u) + np.floor(v)) % 2 == 0
    shade = np.where(check, 0.92, 0.42)
    fog = np.clip((RADIUS - 12) / 170, 0, 1) ** 1.3
    small = atkinson(shade * fog)
    return np.repeat(np.repeat(small, CHUNK, 0), CHUNK, 1)[:H, :W]

# ---- the titles over the tunnel -------------------------------------------------------------------------------------

FUTURA = "/System/Library/Fonts/Supplemental/Futura.ttc"   # Futura Medium, the film's lettering

def title_card(lines, size):
    """White one-bit letters rimmed in black, centred, drawn at the tunnel's 128x86 and shown in its 4x4 blocks:
    (white, black) masks at the screen's size."""
    from PIL import Image, ImageDraw, ImageFont
    f = ImageFont.truetype(FUTURA, size, index=2)
    im = Image.new("L", (TW, TH), 0)
    d = ImageDraw.Draw(im)
    gap = int(size * 1.25)
    y = TH // 2 - gap * len(lines) // 2
    for line in lines:
        w = d.textlength(line, font=f)
        d.text(((TW - w) / 2, y), line, fill=255, font=f)
        y += gap
    white = np.array(im) > 127
    p = np.pad(white, 1)
    rim = white | p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:] | p[:-2, :-2] | p[2:, 2:] | p[:-2, 2:] | p[2:, :-2]
    big = lambda m: np.repeat(np.repeat(m, CHUNK, 0), CHUNK, 1)[:H, :W]
    return big(white), big(rim)

TITLES = [title_card(["STRANGE QUERY", "PRESENTS"], 9), title_card(["A CARDIAC", "PRODUCTION"], 9),
          title_card(["CARD", "ATTACK"], 22)]
DISSOLVE = np.random.default_rng(1987).random((H, W))   # HyperCard's dissolve: each pixel's turn

# ---- the shot -------------------------------------------------------------------------------------------------------

def render(S):
    img = desktop()
    for kind, x, y, name, sel in S["icons"]:
        draw_icon(img, kind, x, y, name, selected=sel)
    for i, w in enumerate(S["windows"]):
        window(img, w, active=(i == len(S["windows"]) - 1))
    if S.get("alert"):
        alert(img, S["alert"])
    menubar(img, S["menus"])
    for rect in S["zoom"]:
        invert_frame(img, *rect)
    if S["black"]:
        img[:] = True
    if S.get("tunnel") is not None:
        img = tunnel(S["tunnel"])
    if S.get("title"):
        which, shown = S["title"]
        white, rim = TITLES[which]
        on = DISSOLVE < shown
        img[rim & on] = True
        img[white & on] = False
    screen_corners(img)
    if S["cursor"]:
        cursor(img, int(round(S["mouse"][0])), int(round(S["mouse"][1])), S["cursor"])
    return img

class Script:
    def __init__(self):
        self.S = {"icons": [["hd", 464, 28, "Hard Disk", False], ["trash", 464, 292, "Trash", False]],
                  "windows": homework_windows(), "menus": [APPLE, "File", "Edit", "View", "Special"],
                  "zoom": [], "black": False, "cursor": "arrow", "mouse": (250.0, 330.0)}
        self.frames = []
        self.marks = []                                   # (frame, what): where the sounds go

    def mark(self, what):
        self.marks.append((len(self.frames), what))

    def snap(self, n=1):
        for _ in range(n):
            self.frames.append(render(self.S))

    def glide(self, to, n=None):
        x0, y0 = self.S["mouse"]
        dx, dy = to[0] - x0, to[1] - y0
        dist = (dx * dx + dy * dy) ** 0.5
        n = max(2, round(1.5 + dist / 110))               # ~3 frames across a window, 5 across the screen
        for k in range(1, n + 1):
            u = 1 - (1 - k / n) ** 3
            bow = 0.08 * (1 - (2 * u - 1) ** 2)           # the wrist's arc, off to one side
            self.S["mouse"] = (x0 + dx * u - dy * bow, y0 + dy * u + dx * bow)
            self.snap()

    def close_front(self, read=16):
        w = self.S["windows"][-1]
        l, t, r, b = w["rect"]
        if w.get("alert"):
            self.snap(6)
            self.alert(ICLOUD_ALERT)
        self.glide((l + 13, t + 9))
        self.snap(1)
        w["pressed"] = True
        self.snap(2)
        w["pressed"] = False
        self.S["windows"].pop()
        if "folder_at" in w:                              # a Finder window zooms back to where it came from
            self.zoom(w["rect"], (*w["folder_at"], w["folder_at"][0] + 31, w["folder_at"][1] + 31))
        self.snap(read)

    def alert(self, spec, read=22):
        self.S["alert"] = dict(spec)
        self.snap(read)
        bl, bt, br, bb = OK_BUTTON
        self.glide(((bl + br) // 2 + 3, (bt + bb) // 2 + 2))
        self.snap(1)
        self.S["alert"]["pressed"] = True
        self.snap(2)
        self.S["alert"] = None
        self.snap(5)

    def zoom(self, a, b, steps=10, per_frame=3):
        """The Finder's ZoomRects: twelve outlines from rect a to rect b, a few drawn in each frame."""
        rects = []
        for k in range(steps + 1):
            u = k / steps
            rects.append(tuple(int(round(p + (q - p) * u)) for p, q in zip(a, b)))
        for k in range(0, len(rects), per_frame):
            self.S["zoom"] = rects[k:k + per_frame]
            self.snap()
        self.S["zoom"] = []

    def double_click(self, icon):
        icon[4] = True
        self.snap(2)
        self.snap(1)

def script():
    s = Script()
    s.S["cursor"], s.S["mouse"] = None, (330.0, 240.0)    # typing: the Mac hides the pointer
    book = s.S["windows"][-1]
    rhythm = [1, 1, 2, 1, 0, 1, 1, 1, 2, 0, 0, 1, 1, 1, 0, 1, 2, 1, 1, 0]
    k = 0
    while book["typed"] < len(BOOK):
        book["typed"] = min(len(BOOK), book["typed"] + rhythm[k % len(rhythm)])
        if BOOK[book["typed"] - 1] == "\n":
            s.snap(3)                                     # a think at the end of a line
        book["caret"] = True
        s.snap()
        k += 1
    for k in range(4):                                    # the Alarm Clock goes off: the Apple flashes
        s.S["menus"][0] = "CLOCK" if k % 2 == 0 else APPLE
        book["caret"] = k % 2 == 1
        s.snap(3)
    s.S["cursor"] = "arrow"
    s.alert(REMINDER, read=16)
    while s.S["windows"]:                                 # and now everything, in a hurry
        s.close_front(read=3 if len(s.S["windows"]) > 1 else 4)
    launch(s)
    return s.frames

def launch(s):
    """From the empty desk: the floppy goes in, mounts, opens; the demo is launched; the tunnel and the titles."""
    s.snap(4)
    s.S["cursor"] = "watch"                               # a floppy goes in: the drive grinds
    s.mark("floppy in: insert clunk, then a read grind (14 frames)")
    s.snap(14)
    disk = ["floppy", 464, 76, "Card Attack", False]
    s.S["icons"].insert(1, disk)
    s.S["cursor"] = "arrow"
    s.snap(6)
    s.glide((480, 92), 9)
    s.snap(3)
    s.mark("double-click the disk: mouse clicks; seek clicks as its window opens")
    s.double_click(disk)
    dw = {"title": "Card Attack", "rect": (60, 70, 452, 262), "info": ("8 items", "391K in disk", "9K available"),
          "draw": icons_in(FLOPPY)}
    s.zoom((464, 76, 495, 107), dw["rect"])
    disk[4] = "open"
    s.S["windows"].append(dw)
    s.snap(12)
    l, t, r, b = dw["rect"]
    ax, ay = FLOPPY_APP
    app = (l + 1 + ax, t + 35 + ay)
    s.glide((app[0] + 16, app[1] + 14), 8)
    s.snap(3)
    def picked(img, cl, ct, cr, cb, active):
        icons_in([i for i in FLOPPY if i[0] != "card"])(img, cl, ct, cr, cb, active)
        draw_icon(img, "card", cl + ax, ct + ay, "Card Attack", selected=True, on_desktop=False)
    dw["draw"] = picked
    s.snap(3)
    s.S["cursor"] = "watch"
    s.mark("launch: a long read grind under the zoom and the watch (to the tunnel)")
    s.zoom((app[0], app[1], app[0] + 31, app[1] + 31), (0, 20, W - 1, H - 1), steps=12)
    s.S["windows"], s.S["icons"] = [], []                 # the application takes the screen
    s.S["menus"] = [APPLE, "File", "Edit"]
    s.S["mouse"] = (256.0, 171.0)
    for k in range(24):                                   # two seconds of the watch
        s.S["cursor"] = f"watch{k // 2}"
        s.snap()
    s.S["cursor"] = None
    s.mark("the tunnel starts (the drive stops)")
    cards = [(0, 24), (1, 24), (2, 36)]                   # (title, frames): two seconds, two seconds, three
    timeline = [None] * 24                                # the tunnel alone first, two seconds
    for which, n in cards:
        for k in range(n):
            fade = min(k + 1, n - k, 3) / 3 if which != 2 else min(k + 1, 3) / 3
            timeline.append((which, fade))
    for k, t in enumerate(timeline):                      # the tunnel, in one bit
        s.S["tunnel"] = (k // 3) * 3 / FPS                # it only manages 4 frames a second
        s.S["title"] = t
        s.snap()

def main():
    out = sys.argv[1]
    frames = script()
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "gray", "-s", f"{W}x{H}",
                           "-r", str(FPS), "-i", "-", "-vf", f"scale={W * 2}:{H * 2}:flags=neighbor",
                           "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "12", "-movflags", "+faststart", out],
                          stdin=subprocess.PIPE)
    for f in frames:
        ff.stdin.write(np.where(f, 0, 255).astype(np.uint8).tobytes())
    ff.stdin.close()
    ff.wait()
    print(len(frames), "frames,", round(len(frames) / FPS, 1), "s")

if __name__ == "__main__":
    main()
