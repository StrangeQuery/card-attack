"""Prototype (design): the Mac cold open's start. A made-up Doom-style shareware
shooter runs in a window on the Mac Classic's 512x342 one-bit screen: textured walls and floor cast per column and
per row, the gun bobbing and firing, a status bar with the player's face (no monsters). The Alarm Clock flashes,
Time for Demosplash!, and the game is closed in a hurry, then the two windows behind it: a pin-up GIF (a public-domain photo) stuck halfway
with NO CARRIER, and a poem for Jenny. Then as before (macdesk.launch): the dev floppy, the demo, the tunnel, the
titles.

    python macdoom.py OUT.mp4 [--name N] [--small OUT_SMALL.mp4] [--audio GAME.wav] [--sfx DIR] [--floppy DIR] [--wav TMP.wav]
                                                     the preview in the Mac's bezel (1536x768, 12 fps)
    python macdoom.py --sheet OUT.png                the still sheet of the game's art
    python macdoom.py --marks                        the sound cues, frame by frame

Every piece of the game's art is drawn here (no id Software art, names or lettering): the wall, floor and ceiling
textures are greys, dithered on screen by a 4x4 ordered pattern (QuickDraw's greys); the gun, the status bar and the
face are one-bit art, Atkinson-dithered once, as a Mac artist would have drawn them in MacPaint.
"""
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw

import macdesk as D


def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


W, H, FPS = D.W, D.H, D.FPS
NAME = "Boom"                                             # the game's name

# ---- the game window ------------------------------------------------------------------------------------------------

GAME_RECT = (31, 34, 480, 325)                            # content 448 x 271: the view 448x224, the status bar 47
VW, VH, SBH = 448, 224, 47
FOCAL = (VW / 2) / 0.8                                    # the screen plane 0.8 wide each side: about 77 degrees
YC = VH / 2
BAYER = (np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]) + 0.5) / 16
BAYER_VIEW = np.tile(BAYER, (VH // 4, VW // 4))

def rng(seed):
    return np.random.default_rng(seed)

# ---- textures: 64x64 greys ------------------------------------------------------------------------------------------

def tex_brick():
    t = np.full((64, 64), 0.18)
    r = rng(1)
    for row in range(4):
        off = 0 if row % 2 == 0 else 16
        for col in range(-1, 3):
            x0, y0 = col * 32 + off, row * 16
            face = 0.62 + r.uniform(-0.1, 0.1)
            for y in range(y0 + 2, y0 + 16):
                for x in range(x0 + 2, x0 + 32):
                    if 0 <= x < 64:
                        t[y, x] = face + (0.18 if y == y0 + 2 else 0) - (0.2 if y == y0 + 15 else 0) \
                                  + (0.12 if x == x0 + 2 else 0) + r.uniform(-0.08, 0.08)
    for _ in range(14):                                   # chips and cracks
        y, x = r.integers(0, 64, 2)
        t[y:y + 2, x:x + r.integers(1, 4)] *= 0.4
    return np.clip(t, 0, 1)

def tex_tech():
    t = np.full((64, 64), 0.55)
    r = rng(2)
    t += r.uniform(-0.04, 0.04, t.shape)
    for y in (0, 31):                                     # two panels a texture, bevelled
        t[y, :] = 0.95; t[y + 1, :] = 0.8; t[y + 30, :] = 0.12; t[y + 29, :] = 0.3
    t[:, 0] = 0.9; t[:, 63] = 0.15
    for y in (10, 20, 42, 52):                            # grooves
        t[y, 4:60] = 0.15; t[y + 1, 4:60] = 0.85
    for y, x in ((5, 5), (5, 58), (26, 5), (26, 58), (36, 5), (36, 58), (57, 5), (57, 58)):
        t[y - 1:y + 2, x - 1:x + 2] = 0.2; t[y - 1, x - 1] = 1.0   # rivets
    t[3:61, 27:37] = 0.08                                 # the light strip down the middle
    t[5:59, 29:35] = 1.0
    t[5:59, 31:33] = 0.85
    return np.clip(t, 0, 1)

def tex_stone():
    r = rng(3)
    pts = r.uniform(0, 64, (14, 2))
    y, x = np.mgrid[0:64, 0:64].astype(float)
    d = []
    for py, px in pts:                                    # Voronoi blocks, wrapped so the texture tiles
        dy = np.minimum(abs(y - py), 64 - abs(y - py)); dx = np.minimum(abs(x - px), 64 - abs(x - px))
        d.append(np.hypot(dy, dx))
    d = np.array(d)
    o = np.sort(d, 0)
    which = np.argmin(d, 0)
    shade = 0.45 + 0.35 * r.uniform(0, 1, len(pts))[which]
    t = shade - 0.25 * np.clip(1 - (o[1] - o[0]) / 6, 0, 1) + r.uniform(-0.07, 0.07, (64, 64))
    t[(o[1] - o[0]) < 1.6] = 0.08
    return np.clip(t, 0, 1)

def tex_door():
    """The door: a heavy plate in two halves, rivets, a big chevron, hazard stripes along its bottom edge."""
    t = np.full((64, 64), 0.72) + rng(4).uniform(-0.04, 0.04, (64, 64))
    y, x = np.mgrid[0:64, 0:64]
    stripes = ((x + y) // 5) % 2 == 0
    t[54:][stripes[54:]] = 0.05; t[54:][~stripes[54:]] = 0.95
    t[53, :] = 0.1
    t[:53, 0:2] = 0.15; t[:53, 62:] = 0.15
    t[4:50, 6:58] = 0.5; t[5:49, 7:57] = 0.78             # the plate, bevelled
    t[4, 6:58] = 0.95; t[49, 6:58] = 0.2
    chev = (y - 14) - abs(x - 32)
    t[(chev >= 0) & (chev < 8) & (y > 12) & (y < 44)] = 0.15   # the chevron, pointing up: it goes up
    t[(chev >= 12) & (chev < 18) & (y > 12) & (y < 44)] = 0.3
    for ry in (8, 45):
        for rx in (10, 54):
            t[ry - 1:ry + 2, rx - 1:rx + 2] = 0.1; t[ry - 1, rx - 1] = 1.0
    return np.clip(t, 0, 1)

def tex_jamb():
    """The door's jambs: the corridor's panels with a band of hazard stripes down the middle."""
    t = tex_tech()
    y, x = np.mgrid[0:64, 0:64]
    band = (x >= 22) & (x < 42)
    t[band & (((x + y) // 6) % 2 == 0)] = 0.08
    t[band & (((x + y) // 6) % 2 == 1)] = 0.92
    t[:, 21] = 0.1; t[:, 42] = 0.1
    return t

def tex_floor():
    t = np.full((64, 64), 0.62) + rng(5).uniform(-0.06, 0.06, (64, 64))
    t[0, :] = 0.15; t[:, 0] = 0.15                        # one big tile a cell
    t[1, :] = 0.85; t[:, 1] = 0.85
    t[30:34, 30:34] = 0.3                                 # a bolt in the middle
    return np.clip(t, 0, 1)

def tex_ceiling():
    t = np.full((64, 64), 0.3) + rng(7).uniform(-0.04, 0.04, (64, 64))
    t[::32, :] = 0.1; t[:, ::32] = 0.1
    t[24:40, 24:40] = 0.06; t[26:38, 26:38] = 1.0         # a light panel
    return np.clip(t, 0, 1)

TEX = {1: tex_brick(), 2: tex_tech(), 3: tex_stone(), 4: tex_jamb(), 5: tex_door()}
FLOOR, CEIL = tex_floor(), tex_ceiling()

# ---- the map --------------------------------------------------------------------------------------------------------


# ---- one-bit art: Atkinson-dithered greys with a mask ---------------------------------------------------------------

def bit_art(grey, mask, outline=True):
    """(ink, mask): Atkinson's dither of the greys inside the mask, the mask's edge inked."""
    g = np.where(mask, grey, 1.0)
    ink = D.atkinson(g) & mask
    if outline:
        ink |= D.edge_of(mask)
    return ink, mask

def canvas(w, h):
    g = Image.new("L", (w, h), 255)
    m = Image.new("L", (w, h), 0)
    return g, m, ImageDraw.Draw(g), ImageDraw.Draw(m)

def done(g, m):
    return np.array(g) / 255.0, np.array(m) > 127

def gun(kick=0, flash=0):
    """The player's pistol from behind and above: the slide's top running away to the front sight, the back of
    the slide with its notch, a gloved fist round the grip with the thumb over. 132x112."""
    w, h = 132, 112
    g, m, dg, dm = canvas(w, h)
    def shape(fn, xy, grey, **k):
        getattr(dg, fn)(xy, fill=int(grey * 255), **k)
        getattr(dm, fn)(xy, fill=255, **k)
    top = 22 - kick
    cx = 66
    if flash:                                             # the muzzle flash, a ragged star
        r1, r2 = (30, 12) if flash == 2 else (18, 8)
        pts = []
        for i in range(16):
            a = math.pi * i / 8 + 0.2
            r = r1 if i % 2 == 0 else r2
            pts.append((cx + r * math.cos(a), top - 4 + r * 0.8 * math.sin(a)))
        shape("polygon", pts, 1.0)
        dg.polygon(pts, fill=255, outline=0)
    top += 10
    back = top + 38                                       # the back of the slide
    shape("polygon", [(cx - 14, top), (cx + 14, top), (cx + 23, back), (cx - 23, back)], 0.35)    # its sides
    dg.polygon([(cx - 10, top), (cx + 10, top), (cx + 16, back), (cx - 16, back)], fill=int(0.82 * 255))  # its top
    shape("ellipse", (cx - 9, top - 4, cx + 9, top + 5), 0.3)      # the muzzle's face
    dg.ellipse((cx - 4, top - 2, cx + 4, top + 3), fill=0)
    dg.line((cx, top + 2, cx, back), fill=int(0.45 * 255), width=2)   # the rib
    shape("rectangle", (cx - 3, top - 9, cx + 3, top - 3), 0.2)      # the front sight
    shape("rectangle", (cx - 24, back, cx + 24, back + 18), 0.22)    # the back plate
    dg.rectangle((cx - 4, back, cx + 4, back + 6), fill=int(0.9 * 255))   # the rear notch
    for k in range(4):                                    # serrations
        dg.line((cx - 22, back + 9 + 2 * k, cx - 10, back + 9 + 2 * k), fill=int(0.6 * 255))
        dg.line((cx + 10, back + 9 + 2 * k, cx + 22, back + 9 + 2 * k), fill=int(0.6 * 255))
    # the gloved fist round the grip: dark, its fingers wrapped across the front, the thumb up the left side
    shape("rounded_rectangle", (cx - 30, back + 14, cx + 32, h + 8), 0.3, radius=12)
    for k in range(3):                                    # the fingers' creases
        y = back + 26 + 9 * k
        dg.line((cx - 24, y, cx + 26, y), fill=int(0.75 * 255), width=1)
    shape("rounded_rectangle", (cx - 38, back - 4, cx - 22, back + 30), 0.55, radius=7)   # the thumb
    dg.line((cx - 34, back + 2, cx - 34, back + 22), fill=int(0.85 * 255), width=1)
    shape("rectangle", (cx - 36, h - 10, cx + 38, h), 0.15)   # the cuff
    return bit_art(*done(g, m))

# ---- the status bar and the face ------------------------------------------------------------------------------------

BIG = {                                                   # a 5x7 grid scaled 3x, emboldened: the counters' digits
    "0": ["01110", "10001", "10011", "10101", "11001", "10001", "01110"],
    "1": ["00100", "01100", "00100", "00100", "00100", "00100", "01110"],
    "2": ["01110", "10001", "00001", "00010", "00100", "01000", "11111"],
    "3": ["11111", "00010", "00100", "00010", "00001", "10001", "01110"],
    "4": ["00010", "00110", "01010", "10010", "11111", "00010", "00010"],
    "5": ["11111", "10000", "11110", "00001", "00001", "10001", "01110"],
    "6": ["00110", "01000", "10000", "11110", "10001", "10001", "01110"],
    "7": ["11111", "00001", "00010", "00100", "01000", "01000", "01000"],
    "8": ["01110", "10001", "10001", "01110", "10001", "10001", "01110"],
    "9": ["01110", "10001", "10001", "01111", "00001", "00010", "01100"],
    "%": ["11001", "11010", "00010", "00100", "01000", "01011", "10011"],
}

def big_glyph(c):
    g = np.array([[ch == "1" for ch in row] for row in BIG[c]]).repeat(3, 0).repeat(3, 1)
    g[:, 1:] |= g[:, :-1]
    return g                                              # 21 x 15

def big_text(img, x, y, s, white=True):
    for c in s:
        gl = big_glyph(c)
        img[y:y + 21, x:x + 15][gl] = not white
        x += 18
    return x

def face(look=0, mood="calm"):
    """The player's face, 28x34, one bit: a buzz cut, a square jaw, eyes left/ahead/right (look -1/0/1)."""
    w, h = 28, 34
    yy, xx = np.mgrid[0:h, 0:w]
    m = ((xx - 13.5) / 11.5) ** 2 + ((yy - 15) / 14.5) ** 2 <= 1
    m |= (yy >= 15) & (yy <= 30) & (abs(xx - 13.5) <= 10.5 - np.maximum(0, yy - 24) * 0.9)   # the jaw, squared
    m[14:21, 1] = m[14:21, 26] = True                     # ears
    ink = D.edge_of(m)
    ink |= m & (yy < 8) & ((xx + yy) % 2 == 0)            # the stubble on top
    ink |= m & (yy < 4)
    ink |= m & (xx >= 21) & (yy >= 10) & ((xx + yy) % 2 == 0)   # the shadow side
    ink |= m & (yy >= 27) & ((xx + yy) % 2 == 0) & (abs(xx - 13.5) > 3)   # jaw stubble
    f = np.zeros((h, w), bool)
    def px(x, y):
        f[y, x] = True
    raised = mood == "hurt"
    for x in range(5, 12):                                # brows, angry: low in the middle
        px(x, 10 + (x - 5) // 4 - (2 if raised else 0))
        px(x, 11 + (x - 5) // 4 - (2 if raised else 0))
    for x in range(16, 23):
        px(x, 11 - (x - 16) // 4 - (2 if raised else 0))
        px(x, 12 - (x - 16) // 4 - (2 if raised else 0))
    for ex in (6, 16):                                    # the eyes
        if mood == "hurt":
            for x in range(ex, ex + 6):
                px(x, 14)
            continue
        ink[13:17, ex:ex + 6] = False
        for x in range(ex, ex + 6):
            px(x, 13); px(x, 16)
        p = ex + 2 + look
        f[14:16, p:p + 2] = True
    for y in range(15, 21):                               # the nose
        px(14 + (1 if y > 18 else 0), y)
    px(12, 21); px(13, 21); px(14, 21); px(15, 21)
    if mood == "grin":                                    # the mouth
        ink[23:27, 8:20] = False
        for x in range(8, 20):
            px(x, 23); px(x, 26)
        for x in range(9, 20, 2):
            px(x, 24); px(x, 25)
        px(7, 22); px(20, 22)
    elif mood == "hurt":
        for x in range(10, 18):
            px(x, 23); px(x, 27)
        for y in range(23, 28):
            px(10, y); px(17, y)
        ink[24:27, 11:17] = True
    else:
        for x in range(9, 19):
            px(x, 24)
        px(8, 23); px(19, 25)
    return ink | (f & m), m

STATUS_BOXES = [("AMMO", 4, 84), ("HEALTH", 90, 190), ("ARMS", 196, 246), (None, 252, 292), ("ARMOR", 298, 398),
                ("KEYS", 404, 443)]

def status_number_x(l, r, t):
    """Where a counter's first digit goes in its box."""
    return (l + r) // 2 - (18 * len(t) - 3) // 2

def status_base():
    """The status bar without its counters and face: the grey, the sunken boxes, the labels, the arms, the key."""
    s = np.zeros((SBH, VW), bool)
    s[(np.add.outer(np.arange(SBH), np.arange(VW)) % 2 == 0) & (np.arange(SBH)[:, None] % 2 == 0)] = True  # ltGray
    s[0, :] = True; s[1, :] = False
    for label, l, r in STATUS_BOXES:
        s[4:SBH - 3, l:r + 1] = True                      # sunken black boxes
        s[SBH - 3, l:r + 1] = False; s[4:SBH - 2, r + 1] = False
        if label:
            wl = D.width(label, D.GENEVA9)
            D.text(s, (l + r) // 2 - wl // 2, SBH - 6, label, D.GENEVA9, ink=False)
    for i, d in enumerate("234567"):                      # the arms: the ones you have lit
        x, y = 203 + (i % 3) * 15, 16 + (i // 3) * 12
        if d in "23":
            D.text(s, x, y, d, D.GENEVA9, ink=False)
        else:
            s[y - 3, x + 1:x + 3] = False
    key = ["..XXX.......", ".X...X......", ".X...XXXXXXX", ".X...X..X.X.", "..XXX.......", ]   # one key
    for j, row in enumerate(key):
        for i, c in enumerate(row):
            if c == "X":
                s[14 + j, 418 + i] = False
    return s

def status_bar(ammo, health, armor, look, mood):
    s = status_base()
    big_text(s, status_number_x(4, 84, str(ammo)), 7, str(ammo))
    big_text(s, status_number_x(90, 190, f"{health}%"), 7, f"{health}%")
    big_text(s, status_number_x(298, 398, f"{armor}%"), 7, f"{armor}%")
    fi, fm = face(look, mood)
    sub = s[8:8 + 34, 258:258 + 28]
    sub[fm] = False
    sub[fi] = True
    return s

# ---- the renderer: per column, a ray; per row, the floor ----------------------------------------------------------

DOOR = 5                                                  # the door's cell code: a wall that rises into the ceiling

def make_map():
    """A corridor (tech panels) to a door between hazard jambs; through it, a brick room with stone pillars."""
    m = np.full((17, 24), 1, int)
    m[7:10, 1:10] = 0                                     # the corridor, three wide
    m[6, 1:10] = 2; m[10, 1:10] = 2; m[7:10, 0] = 2
    m[7, 10] = 4; m[9, 10] = 4                            # the jambs
    m[8, 10] = DOOR
    m[2:15, 11:22] = 0                                    # the room beyond
    m[4:6, 14:16] = 3; m[11:13, 14:16] = 3; m[4:6, 18:20] = 3; m[11:13, 18:20] = 3   # four pillars
    return m

MAP = make_map()

def cast(px, py, a):
    """Every column's ray, stepped cell by cell (Amanatides and Woo): the wall it stops at (distance, which
    texture, where along it, whether it faces north/south) and, if it passed through the door's cell, the door's
    face too, drawn over the far wall as far down as the door has come."""
    dx, dy = math.cos(a), math.sin(a)
    rx, ry = -math.sin(a) * 0.8, math.cos(a) * 0.8        # the screen plane, to the right
    out = []
    for c in range(VW):
        cam = 2 * (c + 0.5) / VW - 1
        rdx, rdy = dx + rx * cam, dy + ry * cam
        mx, my = int(px), int(py)
        ddx = abs(1 / rdx) if rdx else 1e9
        ddy = abs(1 / rdy) if rdy else 1e9
        sx, sdx = (-1, (px - mx) * ddx) if rdx < 0 else (1, (mx + 1 - px) * ddx)
        sy, sdy = (-1, (py - my) * ddy) if rdy < 0 else (1, (my + 1 - py) * ddy)
        door = None
        while True:
            if sdx < sdy:
                sdx += ddx; mx += sx; side = 0
            else:
                sdy += ddy; my += sy; side = 1
            k = MAP[my, mx]
            if k == 0:
                continue
            dist = (sdx - ddx) if side == 0 else (sdy - ddy)
            hit = (py + dist * rdy) if side == 0 else (px + dist * rdx)
            u = hit - math.floor(hit)
            if (side == 0 and rdx < 0) or (side == 1 and rdy > 0):
                u = 1 - u
            if k == DOOR:
                if door is None:
                    door = (dist, u, side)
                continue                                  # see past it, under it
            break
        out.append((dist, k, u, side, door))
    return out

def light_of(dist):
    return np.clip(1.55 - dist / 11.0, 0.1, 1.0)

def wall_column(grey, c, dist, tex, u, side, z_lo=0.0, shift=0.0):
    """One column of a wall from height z_lo up to the ceiling (1), its texture moved up by shift."""
    lh = FOCAL / max(dist, 1e-3)
    top = YC - lh / 2                                     # z = 1
    bot = YC + lh * (0.5 - z_lo)                          # z = z_lo
    y0, y1 = max(0, int(math.ceil(top - 0.5))), min(VH, int(math.ceil(bot - 0.5)))
    if y1 <= y0:
        return
    ys = np.arange(y0, y1) + 0.5
    tv = ((ys - top) / lh * 64 + shift * 64).astype(int).clip(0, 63)
    tu = min(63, int(u * 64))
    grey[y0:y1, c] = tex[tv, tu] * light_of(dist) * (0.8 if side else 1.0)

def view(px, py, a, door_open=0.0, bright=1.0):
    """The 3D view, ink True = black: the flats per row, the walls per column, all as greys through the ordered
    dither (QuickDraw's greys). door_open: how far the door has risen, 0 to 1."""
    grey = np.zeros((VH, VW))
    dirx, diry = math.cos(a), math.sin(a)
    rx, ry = -math.sin(a) * 0.8, math.cos(a) * 0.8
    cam = 2 * (np.arange(VW) + 0.5) / VW - 1
    for y in range(VH):                                   # the floor and the ceiling, per row
        dy = abs(y + 0.5 - YC)
        rd = FOCAL * 0.5 / dy
        wx = px + rd * (dirx + rx * cam)
        wy = py + rd * (diry + ry * cam)
        tx = (wx * 64).astype(int) % 64
        ty = (wy * 64).astype(int) % 64
        below = y + 0.5 > YC
        grey[y] = (FLOOR if below else CEIL)[ty, tx] * light_of(rd)
    for c, (dist, kind, u, side, door) in enumerate(cast(px, py, a)):   # the walls, per column
        wall_column(grey, c, dist, TEX[kind], u, side)
        if door is not None and door_open < 1:
            dd, du, dside = door
            wall_column(grey, c, dd, TEX[DOOR], du, dside, z_lo=door_open, shift=door_open)
    return (grey * bright) <= BAYER_VIEW

# ---- the game's script ----------------------------------------------------------------------------------------------

GAME_FRAMES = 60                                          # 5 s, then the alert
SHOTS = [7, 44, 50]                                       # a shot down the corridor, two into the room
DOOR_AT, DOOR_FRAMES = 15, 10                             # the door starts up as you reach it; open in 10 frames

def lerp(a, b, u):
    return a + (b - a) * u

def smooth(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)

def door_open(f):
    return smooth((f - DOOR_AT) / DOOR_FRAMES)

def player(f):
    """Where the player is and faces at frame f, the walk's phase and how hard the gun bobs."""
    if f <= 15:                                           # down the corridor, at a run, to the door
        x = lerp(1.8, 7.7, smooth(f / 15 * 0.5 + 0.5) * 2 - 1); y = 8.5; a = 0.0; bob = 1.0
    elif f <= 21:                                         # at the door while it goes up
        x = 7.7; y = 8.5; a = 0.0; bob = 0.3
    elif f <= 36:                                         # through it into the room
        u = (f - 21) / 15
        x = lerp(7.7, 13.0, smooth(u * 0.5) * 2); y = 8.5; a = lerp(0.0, 0.15, u); bob = 1.0
    else:                                                 # turning right past the pillars
        u = smooth((f - 36) / 24)
        x = lerp(13.0, 14.6, u); y = lerp(8.5, 9.6, u); a = lerp(0.15, 1.0, u); bob = 0.7
    return x, y, a, f * 0.6, bob

GUNS = {(0, 0): gun(), (8, 2): gun(8, 2), (5, 1): gun(5, 1), (2, 0): gun(2, 0)}

def game_frame(f):
    """The window's content at game frame f: the view and the status bar, 448x271, ink True."""
    px, py, a, phase, speed = player(f)
    kick = 0
    flash = 0
    for s in SHOTS:
        k = f - s
        if k == 0:
            kick, flash = 8, 2
        elif k == 1:
            kick, flash = 5, 1
        elif k == 2:
            kick = 2
    v = view(px, py, a, door_open(f), bright=1.35 if flash == 2 else 1.0)
    gi, gm = GUNS[(kick, flash)]
    gh, gw = gi.shape
    bx = int(round(12 * math.sin(phase) * speed))
    by = int(round(7 * abs(math.cos(phase)) * speed))
    x0, y0 = VW // 2 - gw // 2 + 18 + bx, VH - gh + 10 + by - kick
    for j in range(gh):
        y = y0 + j
        if 0 <= y < VH:
            xs = slice(max(0, x0), min(VW, x0 + gw))
            mrow = gm[j, xs.start - x0:xs.stop - x0]
            v[y, xs][mrow] = gi[j, xs.start - x0:xs.stop - x0][mrow]
    ammo = 50 - sum(1 for s in SHOTS if f >= s)
    look = [0, -1, 0, 1][(f // 9) % 4]
    mood = "grin" if any(s <= f < s + 4 for s in SHOTS) else "calm"
    sb = status_bar(ammo, 100, 50, look, mood)
    return np.vstack([v, sb])

# ---- the other two windows ------------------------------------------------------------------------------------------

GIF_RECT = (12, 24, 234, 310)                             # content 220 x 250 + the status line
GIF_W, GIF_H, GIF_STALL = 220, 250, 128                   # stalled at row 128: 51%

BCHBABE_SRC = ("images", "bchbabe_src.jpg")   # in $CARD_ATTACK_WORK; kept out of git

def pinup_prepared():
    """The photo's greys before the dither (0-255, floats): cropped from the top to 220x250, sharpened, stretched,
    lightened (gamma 0.7)."""
    from PIL import ImageFilter, ImageOps
    im = Image.open(_work(*BCHBABE_SRC)).convert("L")
    w, h = im.size
    ch = int(w * GIF_H / GIF_W)                           # crop from the top: head to thighs
    im = im.crop((0, 0, w, min(h, ch))).resize((GIF_W, GIF_H), Image.LANCZOS)
    im = im.filter(ImageFilter.UnsharpMask(radius=3, percent=160, threshold=2))
    im = ImageOps.autocontrast(im, cutoff=3)
    a = np.asarray(im, np.float32)
    return 255 * (a / 255) ** 0.7                         # a touch lighter: dither reads darker than grey

def pinup_greys():
    """The same greys as whole numbers out of 256 (white), for the build's dither."""
    return np.clip(np.round(pinup_prepared() * 256 / 255), 0, 256).astype(int)

def pinup():
    """BCHBABE.GIF: a public-domain photo, prepared (pinup_prepared) and Atkinson-dithered to one bit (as
    dither_babe2.py does it, exactly). Ink True = black."""
    a = pinup_prepared().copy()
    for y in range(GIF_H):                                # Atkinson: 6/8 of the error to six neighbours
        for x in range(GIF_W):
            o = a[y, x]; n = 255.0 if o >= 128 else 0.0; a[y, x] = n; e = (o - n) / 8
            for dx, dy in ((1, 0), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)):
                if 0 <= x + dx < GIF_W and y + dy < GIF_H:
                    a[y + dy, x + dx] += e
    return ~(a > 127)

PINUP = pinup()                                           # as a GIF viewer on a one-bit Mac shows it


def gif_draw(img, l, t, r, b, active):
    pic = PINUP.copy()
    pic[GIF_STALL + 1:] = False                           # the rest never came
    pic[GIF_STALL, 131:] = False                          # the last row half received
    img[t:t + GIF_H, l:l + GIF_W] = pic[:, :min(GIF_W, r - l + 1)]
    y = t + GIF_H
    D.fill(img, l, y, r, y, True)
    D.text(img, l + 4, y + 12, "23,552 of 46,080 bytes", D.GENEVA9)
    s = "NO CARRIER"
    D.text(img, r - 4 - D.width(s, D.CHICAGO), y + 13, s, D.CHICAGO)
    big = BIG_NO_CARRIER                                  # and big, in the blank below the stall: unmistakable
    bh, bw = big.shape
    top = t + GIF_STALL + 1 + (GIF_H - GIF_STALL - 1 - bh) // 2
    left = l + (GIF_W - bw) // 2
    img[top:top + bh, left:left + bw] |= big

def big_lines(lines, font, scale, gap):
    """Lines of a Mac bitmap font, centred, each pixel scale x scale: a bool array cropped to the ink."""
    rows = []
    for line in lines:
        scratch = np.zeros((H, W), bool)
        D.text(scratch, 0, 20, line, font)
        ys, xs = np.nonzero(scratch)
        rows.append(scratch[ys.min():ys.max() + 1, :xs.max() + 1])   # cropped to the ink (capitals: no descenders)
    width = max(r.shape[1] for r in rows)
    out = []
    for r in rows:
        pad = width - r.shape[1]
        out.append(np.pad(r, ((0, 0), (pad // 2, pad - pad // 2))))
        out.append(np.zeros((gap, width), bool))
    a = np.vstack(out[:-1])
    ys = np.nonzero(a.any(1))[0]
    a = a[ys[0]:ys[-1] + 1]
    return a.repeat(scale, 0).repeat(scale, 1)

BIG_GOURAUD = big_lines(["GOURAUD"], D.CHICAGO, 3, 0)
BIG_NO_CARRIER = big_lines(["NO", "CARRIER"], D.CHICAGO, 4, 3)   # Chicago at 4x, two lines

TRI = [(4, 72), (172, 84), (142, 24)]                     # the tutorial's triangle in its picture (176 x 86)
TRI_TIP = (110.0, 12)                                     # the tip of its normal
TUTORIAL_TEXT = ("Light\neach\ncorner,\nthen\nstep.")

BIG_WIKI = big_lines(["Gouraud shading"], D.CHICAGO, 2, 0)
WIKI_TEXT = ("Gouraud shading, named after\nHenri Gouraud, is a way to\nshade a polygon mesh smoothly:\n"
             "light each vertex, then blend\nthe colours across each face.\n\nContents\n 1 Description\n"
             " 2 Comparison with Phong\n 3 History")

def tutorial_draw(img, l, t, r, b, active):
    """The Gouraud page, laid out like Wikipedia: the title, a rule, the tagline, text left, a picture box right."""
    from PIL import Image, ImageDraw
    big = BIG_WIKI
    bh, bw = big.shape
    img[t + 8:t + 8 + bh, l + 10:l + 10 + bw] |= big
    y = t + 8 + bh + 5
    img[y, l + 10:r - 24] = True                             # the rule under the title
    D.text(img, l + 10, y + 13, "From Wikipedia, the free encyclopedia", D.GENEVA9)
    D.lines_of(WIKI_TEXT, 10, y + 32 - t, step=13, font=D.GENEVA9)(img, l, t, r, b, active)
    # the picture box on the right: a Gouraud-shaded sphere and its caption
    bx0, by0 = r - 196, y + 22
    bwid, bhgt = 170, 182
    img[by0, bx0:bx0 + bwid] = True; img[by0 + bhgt, bx0:bx0 + bwid] = True
    img[by0:by0 + bhgt, bx0] = True; img[by0:by0 + bhgt, bx0 + bwid] = True
    R = 70; cx, cy = bx0 + bwid // 2, by0 + 8 + R
    ys, xs = np.mgrid[-R:R + 1, -R:R + 1].astype(np.float32)
    inside = xs * xs + ys * ys <= R * R
    z = np.sqrt(np.maximum(0, R * R - xs * xs - ys * ys)) / R
    nx, ny = xs / R, ys / R
    light = np.clip(-0.5 * nx - 0.6 * ny + 0.62 * z, 0, 1) * 0.9 + 0.08
    B4 = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]], np.float32).repeat(2, 0).repeat(2, 1)
    thr = (np.tile(B4, (2 * R // 8 + 2, 2 * R // 8 + 2))[:2 * R + 1, :2 * R + 1] + 0.5) / 16
    dots = inside & (light < thr)
    img[cy - R:cy + R + 1, cx - R:cx + R + 1] |= dots
    D.text(img, bx0 + 8, by0 + bhgt - 8, "A Gouraud-shaded sphere", D.GENEVA9)

BIG_FINNISH = big_lines(["LEARN TO SPEAK", "FINNISH"], D.CHICAGO, 2, 6)

def finnish_draw(img, l, t, r, b, active):
    """Learn to Speak Finnish, under the Gouraud page: the words in bold and the flag. Readable across a room."""
    w = r - l + 1
    big = BIG_FINNISH
    bh, bw = big.shape
    img[t + 10:t + 10 + bh, l + (w - bw) // 2:l + (w - bw) // 2 + bw] |= big
    fw = w - 60; fh = fw * 11 // 18                       # the flag, 18:11, white with a blue Nordic cross
    fl, ft = l + 30, t + 10 + bh + 14
    ys, xs = np.mgrid[0:fh, 0:fw]
    cross = ((xs * 18 // fw >= 5) & (xs * 18 // fw < 8)) | ((ys * 11 // fh >= 4) & (ys * 11 // fh < 7))
    blue = ((xs + ys) % 2 == 0) | (ys % 2 == 0)               # blue as a dense pattern: three dots in four
    ch = min(fh, b - 8 - ft)                              # cropped to fit the window, its bottom cut off
    img[ft:ft + ch, fl:fl + fw] = (cross & blue)[:ch]
    img[ft, fl:fl + fw] = True; img[ft + ch - 1, fl:fl + fw] = True
    img[ft:ft + ch, fl] = True; img[ft:ft + ch, fl + fw - 1] = True

def gif_window():
    """The bottom window, under the Gouraud page (it was BCHBABE.GIF): Learn to Speak Finnish, the scene's home tongue."""
    return {"title": "Learn to Speak Finnish", "rect": (70, 64, 420, 296), "scroll": False, "zoom": False, "draw": finnish_draw}

def poem_window():
    """The third window: a coding tutorial in TeachText, the kid learning to make demos."""
    return {"title": "Gouraud shading - Wikipedia", "rect": (24, 26, 496, 318), "draw": tutorial_draw}

# ---- the shot -------------------------------------------------------------------------------------------------------

MENUS = {"game": [D.APPLE, "File", "Edit", "Game", "Options"],
         "term": [D.APPLE, "File", "Edit", "Settings", "Phone", "Transfer"],
         "teach": [D.APPLE, "File", "Edit"],
         "finder": [D.APPLE, "File", "Edit", "View", "Special"]}

def close(s, menus, read):
    """The front window's close box, clicked in a hurry; the menu bar turns to the next application's."""
    w = s.S["windows"][-1]
    l, t, r, b = w["rect"]
    s.glide((l + 13, t + 9))
    s.snap(1)
    w["pressed"] = True
    s.mark(f"close {w['title']}: mouse click")
    s.snap(2)
    w["pressed"] = False
    s.S["windows"].pop()
    s.S["menus"] = list(MENUS[menus])
    s.snap(read)

def script(name=NAME):
    s = D.Script()
    gw = {"title": name, "rect": GAME_RECT, "scroll": False, "zoom": False}
    pic = {"frame": None}
    def game_draw(img, l, t, r, b, active):
        img[t:t + VH + SBH, l:l + VW] = pic["frame"]
    gw["draw"] = game_draw
    s.S["windows"] = [gif_window(), poem_window(), gw]
    s.S["menus"] = list(MENUS["game"])
    s.S["cursor"], s.S["mouse"] = None, (300.0, 200.0)    # the game hides the pointer
    s.mark("the game: the soundtrack starts")
    for f in range(GAME_FRAMES):
        if f in SHOTS:
            s.mark("gunshot")
        if f == DOOR_AT:
            s.mark("the door rises: door-open sound (about 0.8 s)")
        if f == 0:
            s.mark("footsteps while running (to the door, and again through it at frame 22)")
        if f == GAME_FRAMES - 12:
            s.mark("the Alarm Clock goes off: beeps with each flash (4)")
        if f >= GAME_FRAMES - 12:                         # the Alarm Clock flashes while you play on
            k = (f - (GAME_FRAMES - 12)) // 3
            s.S["menus"][0] = "CLOCK" if k % 2 == 0 else D.APPLE
        pic["frame"] = game_frame(f)
        s.snap()
    s.S["menus"][0] = D.APPLE
    s.S["cursor"], s.S["mouse"] = "arrow", (300.0, 230.0)
    s.mark("the reminder: alert chime; the game's music stops dead")
    s.alert(D.REMINDER, read=16)
    close(s, "teach", read=6)                             # half a second on the Gouraud page...
    close(s, "teach", read=8)                             # ...and on Learn to Speak Finnish, under it
    close(s, "finder", read=4)
    D.launch(s)
    return s.frames, s.marks

# ---- the still sheet ------------------------------------------------------------------------------------------------

def sheet(out):
    from PIL import ImageFont
    pad = 12
    tiles = []
    def add(arr, label, scale=2, ink=False):
        a = arr if not ink else ~arr
        tiles.append((np.array(a, float) if not ink else a.astype(float), label, scale))
    for k, t in TEX.items():
        add((t <= np.tile(BAYER, (16, 16))).__invert__(), ["brick", "tech", "stone", "jamb", "door"][k - 1], 3)
    add(~(FLOOR <= np.tile(BAYER, (16, 16))), "floor", 3)
    add(~(CEIL <= np.tile(BAYER, (16, 16))), "ceiling", 3)
    for key, (gi, gm) in GUNS.items():
        add(~gi, f"gun kick {key[0]} flash {key[1]}", 2)
    for look, mood in ((-1, "calm"), (0, "calm"), (1, "calm"), (0, "grin")):
        add(~face(look, mood)[0], f"face {mood} {look:+d}", 4)
    add(~status_bar(46, 79, 50, 0, "calm"), "status bar", 2)
    p = PINUP.copy(); p[GIF_STALL + 1:] = False
    add(~p, "BCHBABE.GIF as stalled", 2)
    for nm in [NAME]:
        tb = np.zeros((26, 220), bool)
        img = np.zeros((H, W), bool)
        D.window(img, {"title": nm, "rect": (0, 0, 219, 25), "scroll": False, "zoom": False,
                       "draw": lambda *a: None}, active=True)
        add(~img[:26, :221], f"name: {nm}", 2)
    sample = np.zeros((H, W), bool)
    sample[:] = render_one(19)
    add(~sample, "the screen at game frame 19: the door going up", 1)
    # lay out in rows up to 1600 wide
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Futura.ttc", 14, index=0)
    rows, cur, wsum = [], [], 0
    for a, label, sc in tiles:
        tw = a.shape[1] * sc + pad
        if wsum + tw > 1600 and cur:
            rows.append(cur); cur, wsum = [], 0
        cur.append((a, label, sc)); wsum += tw
    rows.append(cur)
    hgt = sum(max(a.shape[0] * sc for a, _, sc in r) + 30 + pad for r in rows) + pad
    im = Image.new("RGB", (1600 + pad, hgt), (200, 200, 200))
    d = ImageDraw.Draw(im)
    y = pad
    for r in rows:
        x = pad
        rh = max(a.shape[0] * sc for a, _, sc in r)
        for a, label, sc in r:
            t = Image.fromarray((a * 255).astype(np.uint8)).resize((a.shape[1] * sc, a.shape[0] * sc), Image.NEAREST)
            im.paste(t.convert("RGB"), (x, y))
            d.text((x, y + rh + 6), label, fill=(0, 0, 0), font=font)
            x += a.shape[1] * sc + pad
        y += rh + 30 + pad
    im.save(out)

def render_one(f, name=NAME):
    s = D.Script()
    gw = {"title": name, "rect": GAME_RECT, "scroll": False, "zoom": False}
    fr = game_frame(f)
    gw["draw"] = lambda img, l, t, r, b, active: img.__setitem__((slice(t, t + VH + SBH), slice(l, l + VW)), fr)
    s.S["windows"] = [gif_window(), poem_window(), gw]
    s.S["menus"] = list(MENUS["game"])
    s.S["cursor"] = None
    return D.render(s.S)

# ---- the preview ----------------------------------------------------------------------------------------------------

def audio_track(marks, n, game_wav, sfx_dir, floppy_dir, out_wav):
    """The preview's sound, 48 kHz mono: the game's soundtrack under the game, cut dead as the reminder comes up;
    the effects (sfx_dir) and the floppy drive (floppy_dir) at their cues. Returns the cue sheet."""
    import wave
    sr = 48000
    total = np.zeros(int(n / FPS * sr) + sr, np.float32)
    def load(p):
        with wave.open(p) as w:
            ch, rate = w.getnchannels(), w.getframerate()
            a = np.frombuffer(w.readframes(w.getnframes()), np.int16).astype(np.float32) / 32768
        assert rate == sr, p
        return a.reshape(-1, ch).mean(1)
    sheet = []
    def put(path, frame, gain=1.0, seconds_early=0.0, until=None):
        if path is None or not os.path.exists(path):
            return
        a = load(path)
        i = max(0, int((frame / FPS - seconds_early) * sr))
        if until is not None:
            a = a[:max(0, int(until / FPS * sr) - i)].copy()
            a[-480:] *= np.linspace(1, 0, min(480, len(a)))
        a = a[:len(total) - i]
        total[i:i + len(a)] += a * gain
        sheet.append((i / sr, os.path.basename(path), len(a) / sr))
    sfx = (lambda nm: os.path.join(sfx_dir, nm)) if sfx_dir else (lambda nm: None)
    flp = (lambda nm: os.path.join(floppy_dir, nm)) if floppy_dir else (lambda nm: None)
    at = {}
    for f, w in marks:
        at.setdefault(w.split(":")[0].split(" (")[0], f)
    if game_wav:
        put(game_wav, 0, 0.7, until=at["the reminder"])
    for f in list(range(0, 16, 4)) + list(range(22, 37, 4)):   # running: a step every third of a second
        put(sfx("step.wav"), f, 0.35)
    for f, w in marks:
        if w.startswith("gunshot"):
            put(sfx("gunshot.wav"), f, 0.8)
        elif w.startswith("the door rises"):
            put(sfx("door.wav"), f, 0.8)
        elif w.startswith("the Alarm Clock"):
            for k in range(4):
                put(sfx("beep.wav"), f + 3 * k, 0.35)
        elif w.startswith("the reminder"):
            put(sfx("chime.wav"), f, 0.5)
        elif w.startswith("close") or w.startswith("double-click"):
            put(sfx("click.wav"), f - 1, 0.5)
    fin = at.get("floppy in")
    if fin is not None:                                   # the disk slides in; its clunk lands as the watch comes
        put(flp("insert.wav"), fin, 0.8, seconds_early=1.91)
        put(flp("grind_short.wav"), fin + 4, 0.6)
    if "double-click the disk" in at:
        put(flp("seek.wav"), at["double-click the disk"] + 3, 0.6)
    if "launch" in at:
        put(flp("grind_long.wav"), at["launch"], 0.6, until=at.get("the tunnel starts"))
    total = np.clip(total, -1, 1)
    with wave.open(out_wav, "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(sr)
        w.writeframes((total * 32767).astype(np.int16).tobytes())
    return sorted(sheet)

def main():
    args = sys.argv[1:]
    def opt(k):
        if k in args:
            i = args.index(k); v = args[i + 1]; del args[i:i + 2]; return v
        return None
    if "--marks" in args:
        frames, marks = script()
        for f, w in marks:
            print(f"{f:4d}  {f / FPS:5.2f} s  {w}")
        print(len(frames), "frames,", round(len(frames) / FPS, 2), "s")
        return
    if "--sheet" in args:
        sheet(args[args.index("--sheet") + 1])
        return
    name = opt("--name") or NAME
    small = opt("--small")
    game_wav = opt("--audio")
    floppy = opt("--floppy")
    sfx = opt("--sfx")
    out = args[0]
    import macpc as P
    frames, marks = script(name)
    for f, w in marks:
        print(f"{f:4d}  {f / FPS:5.2f} s  {w}")
    wav = None
    if game_wav or floppy or sfx:
        wav = opt("--wav") or os.path.splitext(out)[0] + "_audio.wav"
        for t, nm, ln in audio_track(marks, len(frames), game_wav, sfx, floppy, wav):
            print(f"  sound {t:6.2f} s  {nm} ({ln:.2f} s)")
    bezel = P.render(0)
    enc = ["-c:v", "libx264", "-profile:v", "main", "-level", "4.0", "-bf", "0", "-g", "12", "-pix_fmt", "yuv420p",
           "-movflags", "+faststart"]
    ain = ["-i", wav] if wav else []
    aenc = ["-c:a", "aac", "-b:a", "160k", "-shortest"] if wav else []
    def ff(path, scale, crf):
        vf = ["-vf", f"scale={P.W // scale}:{P.H // scale}:flags=neighbor"] if scale > 1 else []
        return subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24",
                                 "-s", f"{P.W}x{P.H}", "-r", str(FPS), "-i", "-"] + ain + vf + enc + ["-crf", str(crf)]
                                + aenc + [path], stdin=subprocess.PIPE)
    outs = [ff(out, 1, 16)] + ([ff(small, 2, 20)] if small else [])
    for fr in frames:
        img = bezel.copy()
        mac = np.where(fr, 0, 255).astype(np.uint8).repeat(2, 0).repeat(2, 1)
        img[42:42 + 684, 256:256 + 1024] = mac[..., None]
        b = img.tobytes()
        for p in outs:
            p.stdin.write(b)
    for p in outs:
        p.stdin.close(); p.wait()
    print(len(frames), "frames,", round(len(frames) / FPS, 1), "s")

if __name__ == "__main__":
    main()
