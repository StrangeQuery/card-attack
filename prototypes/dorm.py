"""The dorm room: halfway through Mode X's mirror ball the camera pulls back out of the monitor
into a cramped dorm room: a 17-inch CRT and a tower on a desk against the back wall, a bunk bed along the left wall,
four people on a couch in front of us watching it from behind (static silhouettes), the room lit by the screen;
Dope starts on the monitor while we're still out, and we dive back in. A flat picture (drawn once, in perspective),
not 3D.

    python prototypes/dorm.py preview OUT.mp4     # the animatic over the cut's own modex and polys frames

The room is laid out in metres and projected once through the camera (the eye 1.45 m up, f 1000 pixels): the back
wall 4.2 m away, 2.6 m wide. It is drawn in final-frame units (1536 x 768) twice: all of it at ART = 2x, and the
monitor's surroundings (INSET) at 8x for the start of the pull-back, where the zoom is about 12. Layers: albedo
(RGB, as lit by white light 1), glow (how much of the monitor's light reaches it), rim (the silhouettes' edges),
emissive (RGB: the lava lamp, the LEDs), the screen's mask. A frame: albedo (AMB + glow x the screen's mean colour x
GAIN) + rim x the mean x RIM + emissive, the screen's own pixels inside the mask; the camera a zoom about a moving
centre, from the screen filling the frame's middle 1024 x 768 (the modex frame's 4:3 picture exactly) out to the
room, and back in to Dope's 2:1 picture (letterboxed on the screen) filling the frame.
"""
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
W, H = 1536, 768
ART, ART_IN = 2, 8
SS = 2                      # supersampling while drawing
F, HZ, EY = 1000.0, 290.0, 1.45      # the camera: focal length (pixels), the horizon's row, the eye's height (m)
AMB = np.array([0.03, 0.035, 0.055])
GAIN = 2.6
RIM = 0.7                   # the silhouettes' edges, lit by the screen
HEL = "/System/Library/Fonts/HelveticaNeue.ttc"


def P(X, Y, d):
    """A point X across, Y up (m), d ahead of the eye: final-frame units."""
    return (W / 2 + F * X / d, HZ - F * (Y - EY) / d)


# the monitor: 0.42 x 0.38 m, its face 3.85 m off, its picture 0.32 x 0.24 m, centred 0.15 m right, 0.94 m up
MON_X, MON_D = 0.15, 3.85
_a, _b = P(MON_X - 0.16, 1.06, MON_D), P(MON_X + 0.16, 0.82, MON_D)
SCREEN = (_a[0], _a[1], _b[0], _b[1])
INSET = (SCREEN[0] - 140, SCREEN[1] - 90, SCREEN[2] + 140, SCREEN[3] + 90)


def coeffs(dst, src):
    """PIL's PERSPECTIVE coefficients taking the output quad `dst` to the input quad `src`."""
    A, B = [], []
    for (x, y), (u, v) in zip(dst, src):
        A += [[x, y, 1, 0, 0, 0, -u * x, -u * y], [0, 0, 0, x, y, 1, -v * x, -v * y]]
        B += [u, v]
    return np.linalg.solve(np.array(A, float), np.array(B, float)).tolist()


RENDERS = os.path.join(ROOT, "segments/rooms/data/dorm3d")
S_MON, S_CONST = 3.15, 0.58       # the renders' light to the layers' (set against the flat drawing's brightness)


def draw_art(scale, box):
    """The layers over `box` from the 3D renders (prototypes/dorm3d/room.py: the whole frame at ART, the INSET at
    ART_IN): albedo the diffuse colour; glow the monitor's light over it (white light 1 through GAIN); emissive the
    rest of the light (the hallway, the lava lamp, the LEDs); the screen where the picture's object is."""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "dorm3d"))
    import exr
    pre = "full" if tuple(box) == (0, 0, W, H) else "inset"
    m = exr.read(os.path.join(RENDERS, f"{pre}_mon.exr"))
    c = exr.read(os.path.join(RENDERS, f"{pre}_const.exr"))
    alb = np.clip(m["Diffuse Color"][..., :3].astype(float), 0, 1)
    asum = alb.sum(-1)
    glow = np.where(asum > 0.003, m["Combined"][..., :3].sum(-1) * S_MON / np.maximum(asum, 1e-6) / GAIN, 0.0)
    glow = np.minimum(glow, 1.5 / np.maximum(alb.max(-1), 1e-3) / GAIN)    # the bezel by the light: B at most 1.5
    out = {"alb": srgb(alb), "glow": glow, "emi": srgb(c["Combined"][..., :3] * S_CONST / 0.9),
           "screen": (np.abs(m["Object Index.X"].reshape(alb.shape[:2]) - 1) < 0.5).astype(float),
           "box": box, "scale": scale}
    out["rim"] = np.zeros_like(glow)
    return out


def draw_art_flat(scale, box):
    """The flat drawing (superseded by the 3D renders): the layers over `box` (final-frame units) at
    `scale` art pixels per unit."""
    bx0, by0, bx1, by1 = box
    K = scale * SS
    w, h = round((bx1 - bx0) * K), round((by1 - by0) * K)
    def k(*v):
        return [round((x - (bx0 if i % 2 == 0 else by0)) * K) for i, x in enumerate(v)]
    def q(*pts):                       # a polygon of room points (X, Y, d)
        return k(*[c for p in pts for c in P(*p)])
    alb = Image.new("RGB", (w, h), (0, 0, 0))
    glw = Image.new("L", (w, h), 0)      # the surfaces' share of the monitor's light (x 1/255)
    emi = Image.new("RGB", (w, h), (0, 0, 0))
    people = Image.new("L", (w, h), 0)
    couch = Image.new("L", (w, h), 0)
    da, dg, de, dp, dc = (ImageDraw.Draw(x) for x in (alb, glw, emi, people, couch))
    def face(pts, col, g):
        da.polygon(q(*pts), fill=col); dg.polygon(q(*pts), fill=g)
    def decal(img, pts, g):
        """An RGBA picture laid on the room quad pts (top left, top right, bottom right, bottom left)."""
        dst = [tuple(q(pt)) for pt in pts]
        iw, ih = img.size
        warped = img.transform((w, h), Image.PERSPECTIVE, coeffs(dst, [(0, 0), (iw, 0), (iw, ih), (0, ih)]), Image.BICUBIC)
        alb.paste(warped.convert("RGB"), (0, 0), warped.split()[3])
        glw.paste(g, (0, 0), warped.split()[3])
    # the right wall moved out to fit the door
    BD, L_, R_, CEIL = 4.2, -1.3, 1.7, 2.4
    # the doorway in the right wall, d DO0..DO1, 2.05 m high; the hallway beyond it, lit by its own tubes (emissive,
    # kept to the doorway's opening at the end)
    DO0, DO1, DH, HW_ = 2.5, 3.4, 2.05, 1.8
    hall = Image.new("RGB", (w, h), (0, 0, 0)); dh = ImageDraw.Draw(hall)
    def hface(pts, col):
        dh.polygon(q(*pts), fill=col)
    hface([(R_, 0, 1.0), (R_ + HW_, 0, 1.0), (R_ + HW_, 0, 12), (R_, 0, 12)], (74, 70, 62))                       # floor
    hface([(R_, CEIL, 1.0), (R_ + HW_, CEIL, 1.0), (R_ + HW_, CEIL, 12), (R_, CEIL, 12)], (88, 88, 84))          # ceiling
    hface([(R_ + HW_, CEIL, 1.0), (R_ + HW_, CEIL, 12), (R_ + HW_, 0, 12), (R_ + HW_, 0, 1.0)], (150, 146, 128))  # far wall
    hface([(R_ + HW_, 0.1, 1.0), (R_ + HW_, 0.1, 12), (R_ + HW_, 0, 12), (R_ + HW_, 0, 1.0)], (60, 56, 50))      # skirting
    for d0 in (2.2, 4.4, 6.6):                                                                                 # the tubes
        hface([(R_ + 0.7, CEIL, d0), (R_ + 1.1, CEIL, d0), (R_ + 1.1, CEIL, d0 + 1.0), (R_ + 0.7, CEIL, d0 + 1.0)], (250, 250, 236))
    hface([(R_ + HW_, 2.05, 3.3), (R_ + HW_, 2.05, 4.2), (R_ + HW_, 0, 4.2), (R_ + HW_, 0, 3.3)], (96, 70, 46))   # a door across
    hface([(R_ + HW_, 1.55, 3.5), (R_ + HW_, 1.55, 3.75), (R_ + HW_, 1.4, 3.75), (R_ + HW_, 1.4, 3.5)], (210, 210, 200))
    hface([(R_ + HW_, 1.6, 4.6), (R_ + HW_, 1.6, 5.4), (R_ + HW_, 1.05, 5.4), (R_ + HW_, 1.05, 4.6)], (120, 96, 60))  # a corkboard
    for i, c in enumerate(((230, 80, 80), (240, 220, 90), (90, 160, 230), (240, 240, 240))):
        d0 = 4.68 + i * 0.17
        hface([(R_ + HW_, 1.5 - (i % 2) * 0.1, d0), (R_ + HW_, 1.5 - (i % 2) * 0.1, d0 + 0.12), (R_ + HW_, 1.32 - (i % 2) * 0.1, d0 + 0.12), (R_ + HW_, 1.32 - (i % 2) * 0.1, d0)], c)
    opening = Image.new("L", (w, h), 0)
    ImageDraw.Draw(opening).polygon(q((R_, DH, DO0), (R_, DH, DO1), (R_, 0, DO1), (R_, 0, DO0)), fill=255)
    # the room: ceiling, the side walls, the floor, the back wall (out to the frame's edges, d 0.9)
    face([(L_, CEIL, 0.9), (R_, CEIL, 0.9), (R_, CEIL, BD), (L_, CEIL, BD)], (70, 72, 82), 150)
    face([(L_, CEIL, 0.9), (L_, CEIL, BD), (L_, 0, BD), (L_, 0, 0.9)], (84, 92, 110), 170)
    for (d0, d1, y0) in ((0.9, DO0, 0), (DO1, BD, 0), (DO0, DO1, DH)):
        face([(R_, CEIL, d0), (R_, CEIL, d1), (R_, y0, d1), (R_, y0, d0)], (84, 92, 110), 170)
    for (d0, d1, y0, y1) in ((DO0 - 0.06, DO0, 0, DH + 0.06), (DO1, DO1 + 0.06, 0, DH + 0.06), (DO0 - 0.06, DO1 + 0.06, DH, DH + 0.06)):
        face([(R_ - 0.005, y1, d0), (R_ - 0.005, y1, d1), (R_ - 0.005, y0, d1), (R_ - 0.005, y0, d0)], (176, 170, 150), 170)
    face([(L_, 0, 0.9), (R_, 0, 0.9), (R_, 0, BD), (L_, 0, BD)], (52, 44, 46), 110)
    face([(L_, CEIL, BD), (R_, CEIL, BD), (R_, 0, BD), (L_, 0, BD)], (92, 98, 118), 255)
    # the ceiling light (off), a poster on the back wall (a demo party's) and one on the right wall (a ray-traced
    # sphere over a checkerboard)
    face([(-0.2, CEIL, 2.6), (0.2, CEIL, 2.6), (0.2, CEIL, 3.0), (-0.2, CEIL, 3.0)], (150, 150, 140), 150)
    PD0, PD1 = 3.55, 4.1
    face([(R_, 1.95, PD0), (R_, 1.95, PD1), (R_, 1.2, PD1), (R_, 1.2, PD0)], (20, 20, 30), 200)
    for i in range(6):
        for j in range(2):
            if (i + j) % 2 == 0:
                d0, d1 = PD0 + i * (PD1 - PD0) / 6, PD0 + (i + 1) * (PD1 - PD0) / 6
                face([(R_, 1.45 - j * 0.12, d0), (R_, 1.45 - j * 0.12, d1), (R_, 1.33 - j * 0.12, d1), (R_, 1.33 - j * 0.12, d0)], (200, 200, 200), 200)
    cxp, cyp = P(R_, 1.7, (PD0 + PD1) / 2)
    rr = F * 0.17 / ((PD0 + PD1) / 2)
    da.ellipse(k(cxp - rr * 0.55, cyp - rr, cxp + rr * 0.55, cyp + rr), fill=(190, 60, 60))
    # the bunk bed along the left wall: 0.9 m wide, 2 m long from the back wall; posts, the two frames, mattresses
    # and blankets, the pillows at the back, the ladder at the front
    BX0, BX1, BN, BF = L_, -0.4, 2.2, BD
    for (y0, y1, bl) in ((0.28, 0.5, (40, 70, 120)), (1.3, 1.52, (120, 40, 50))):
        face([(BX0, y1, BF), (BX1, y1, BF), (BX1, y1, BN), (BX0, y1, BN)], bl, 200)            # the blanket on top
        face([(BX1, y1, BF), (BX1, y1, BN), (BX1, y0, BN), (BX1, y0, BF)], (90, 70, 50), 180)   # the frame's side
        face([(BX0, y1, BN), (BX1, y1, BN), (BX1, y0, BN), (BX0, y0, BN)], (100, 78, 56), 160)  # its end
        face([(BX0 + 0.1, y1 + 0.12, BF - 0.05), (BX1 - 0.15, y1 + 0.12, BF - 0.05), (BX1 - 0.15, y1, BF - 0.45), (BX0 + 0.1, y1, BF - 0.45)], (210, 210, 200), 220)
    for (x, d) in ((BX1, BN), (BX1, BF), (BX0 + 0.02, BN)):
        face([(x - 0.03, 1.85, d), (x + 0.03, 1.85, d), (x + 0.03, 0, d), (x - 0.03, 0, d)], (110, 84, 58), 170)
    for i in range(4):
        y = 0.65 + i * 0.25
        face([(BX1 - 0.02, y + 0.03, BN - 0.02), (BX1 - 0.02, y + 0.03, BN - 0.32), (BX1 - 0.02, y, BN - 0.32), (BX1 - 0.02, y, BN - 0.02)], (120, 92, 64), 170)
    face([(BX1 - 0.02, 1.85, BN - 0.3), (BX1 + 0.02, 1.85, BN - 0.3), (BX1 + 0.02, 0, BN - 0.3), (BX1 - 0.02, 0, BN - 0.3)], (110, 84, 58), 170)
    # the tower on the floor under the desk's right end: its top, its left side, its face, the bays,
    # the floppy slot, the LEDs (the desk's front and leg in front of it)
    tx0, tx1, TF, TB, TT = 0.5, 0.7, BD - 0.48, BD - 0.04, 0.44
    face([(tx0, TT, TF), (tx1, TT, TF), (tx1, TT, TB), (tx0, TT, TB)], (200, 192, 166), 220)
    face([(tx0, TT, TF), (tx0, TT, TB), (tx0, 0, TB), (tx0, 0, TF)], (150, 142, 120), 190)
    face([(tx0, TT, TF), (tx1, TT, TF), (tx1, 0, TF), (tx0, 0, TF)], (190, 182, 155), 230)
    for i in range(3):
        y = TT - 0.06 - i * 0.05
        face([(tx0 + 0.02, y, TF), (tx1 - 0.02, y, TF), (tx1 - 0.02, y - 0.035, TF), (tx0 + 0.02, y - 0.035, TF)], (150, 144, 122), 230)
    face([(tx0 + 0.05, TT - 0.24, TF), (tx1 - 0.05, TT - 0.24, TF), (tx1 - 0.05, TT - 0.25, TF), (tx0 + 0.05, TT - 0.25, TF)], (60, 60, 60), 230)
    for (x, c) in ((tx0 + 0.03, (255, 40, 30)), (tx0 + 0.06, (40, 255, 60))):
        lx, ly = P(x, TT - 0.3, TF); r = F * 0.006 / TF
        de.ellipse(k(lx - r, ly - r, lx + r, ly + r), fill=c)
    # a pair of purple socks on the tower, one over its front edge, one over its side
    face([(tx0 + 0.03, TT + 0.012, TF + 0.18), (tx0 + 0.09, TT + 0.012, TF + 0.18), (tx0 + 0.1, TT + 0.012, TF + 0.01), (tx0 + 0.04, TT + 0.012, TF + 0.01)], (124, 64, 172), 220)
    face([(tx0 + 0.04, TT + 0.01, TF - 0.004), (tx0 + 0.1, TT + 0.01, TF - 0.004), (tx0 + 0.1, TT - 0.1, TF - 0.004), (tx0 + 0.12, TT - 0.13, TF - 0.004), (tx0 + 0.11, TT - 0.16, TF - 0.004), (tx0 + 0.06, TT - 0.15, TF - 0.004), (tx0 + 0.04, TT - 0.1, TF - 0.004)], (124, 64, 172), 230)
    face([(tx0 + 0.04, TT - 0.13, TF - 0.005), (tx0 + 0.1, TT - 0.13, TF - 0.005), (tx0 + 0.1, TT - 0.145, TF - 0.005), (tx0 + 0.05, TT - 0.145, TF - 0.005)], (90, 44, 130), 230)   # its toe
    face([(tx0 + 0.12, TT + 0.014, TF + 0.3), (tx0 + 0.17, TT + 0.014, TF + 0.24), (tx0 + 0.12, TT + 0.014, TF + 0.12), (tx0 + 0.06, TT + 0.014, TF + 0.2)], (146, 76, 196), 220)
    face([(tx0 - 0.003, TT + 0.01, TF + 0.2), (tx0 - 0.003, TT + 0.01, TF + 0.28), (tx0 - 0.003, TT - 0.12, TF + 0.27), (tx0 - 0.003, TT - 0.14, TF + 0.22), (tx0 - 0.003, TT - 0.1, TF + 0.19)], (146, 76, 196), 200)
    # a little wastepaper basket by the desk's left leg, overflowing with napkins and trash
    WX, WD, WR, WHT = -0.36, BD - 0.38, 0.12, 0.3
    face([(WX - WR, WHT, WD), (WX + WR, WHT, WD), (WX + WR * 0.8, 0, WD), (WX - WR * 0.8, 0, WD)], (64, 64, 70), 170)
    for i in range(1, 8):                                      # its wires
        u = -1 + i / 4
        face([(WX + WR * u - 0.004, WHT, WD - 0.001), (WX + WR * u + 0.004, WHT, WD - 0.001), (WX + WR * 0.8 * u + 0.004, 0, WD - 0.001), (WX + WR * 0.8 * u - 0.004, 0, WD - 0.001)], (110, 110, 118), 170)
    face([(WX - WR, WHT + 0.012, WD - 0.002), (WX + WR, WHT + 0.012, WD - 0.002), (WX + WR, WHT - 0.008, WD - 0.002), (WX - WR, WHT - 0.008, WD - 0.002)], (120, 120, 128), 170)  # its rim
    tr = np.random.default_rng(5)
    for i in range(12):                                        # crumpled napkins heaped over the rim
        cxw = WX + (tr.random() * 2 - 1) * WR * 1.05
        cyw = WHT + 0.02 + tr.random() ** 1.5 * 0.13
        rw = 0.03 + tr.random() * 0.025
        col = ((236, 234, 226), (222, 220, 210), (246, 244, 238), (214, 206, 190))[i % 4]
        face([(cxw + rw * (0.7 + 0.5 * tr.random()) * math.cos(2 * math.pi * j / 7), cyw + rw * 0.8 * (0.7 + 0.5 * tr.random()) * math.sin(2 * math.pi * j / 7), WD - 0.01) for j in range(7)], col, 170)
    face([(WX + 0.02, WHT + 0.09, WD - 0.02), (WX + 0.06, WHT + 0.1, WD - 0.02), (WX + 0.07, WHT + 0.03, WD - 0.02), (WX + 0.03, WHT + 0.02, WD - 0.02)], (200, 30, 40), 170)   # a crushed can
    face([(WX - 0.1, WHT + 0.06, WD - 0.025), (WX - 0.02, WHT + 0.09, WD - 0.025), (WX - 0.015, WHT + 0.075, WD - 0.025), (WX - 0.095, WHT + 0.045, WD - 0.025)], (190, 120, 50), 170)  # a crust
    face([(WX + 0.14, 0.05, WD - 0.1), (WX + 0.2, 0.05, WD - 0.1), (WX + 0.2, 0.0, WD - 0.1), (WX + 0.14, 0.0, WD - 0.1)], (236, 234, 226), 160)   # one that missed
    # the desk against the back wall: its top 0.74 m, 1.1 m wide, 0.55 deep; its front, the legs
    DX0, DX1, DD = -0.3, 0.8, BD - 0.55
    face([(DX0, 0.74, DD), (DX1, 0.74, DD), (DX1, 0.74, BD), (DX0, 0.74, BD)], (120, 84, 52), 255)
    face([(DX0, 0.74, DD), (DX1, 0.74, DD), (DX1, 0.66, DD), (DX0, 0.66, DD)], (80, 54, 34), 200)
    for x in (DX0 + 0.02, DX1 - 0.05):
        face([(x, 0.66, DD), (x + 0.04, 0.66, DD), (x + 0.04, 0, DD), (x, 0, DD)], (66, 44, 28), 180)
    # the monitor: its top and side in perspective, its face, the bezel's slope, the foot, its badge and LED
    mx0, mx1 = MON_X - 0.21, MON_X + 0.21
    face([(mx0, 1.14, MON_D), (mx1, 1.14, MON_D), (mx1 - 0.08, 1.12, MON_D + 0.33), (mx0 + 0.08, 1.12, MON_D + 0.33)], (176, 168, 142), 255)
    face([(mx0, 1.14, MON_D), (mx0, 0.78, MON_D), (mx0 + 0.08, 0.80, MON_D + 0.33), (mx0 + 0.08, 1.12, MON_D + 0.33)], (150, 142, 120), 230)
    face([(mx0, 1.14, MON_D), (mx1, 1.14, MON_D), (mx1, 0.78, MON_D), (mx0, 0.78, MON_D)], (196, 188, 160), 255)
    face([(MON_X - 0.18, 1.09, MON_D), (MON_X + 0.18, 1.09, MON_D), (MON_X + 0.18, 0.80, MON_D), (MON_X - 0.18, 0.80, MON_D)], (150, 144, 120), 255)
    face([(MON_X - 0.08, 0.78, MON_D + 0.05), (MON_X + 0.08, 0.78, MON_D + 0.05), (MON_X + 0.08, 0.74, MON_D + 0.05), (MON_X - 0.08, 0.74, MON_D + 0.05)], (170, 162, 138), 255)
    face([(MON_X - 0.15, 0.795, MON_D), (MON_X - 0.08, 0.795, MON_D), (MON_X - 0.08, 0.785, MON_D), (MON_X - 0.15, 0.785, MON_D)], (120, 114, 96), 255)
    lx, ly = P(MON_X + 0.17, 0.79, MON_D); r = F * 0.006 / MON_D
    de.ellipse(k(lx - r, ly - r, lx + r, ly + r), fill=(40, 255, 60))
    # the keyboard, a soda can, the lava lamp (left of the monitor)
    face([(MON_X - 0.22, 0.755, DD + 0.03), (MON_X + 0.22, 0.755, DD + 0.03), (MON_X + 0.22, 0.74, DD + 0.2), (MON_X - 0.22, 0.74, DD + 0.2)], (180, 172, 148), 230)
    face([(-0.12, 0.86, DD + 0.15), (-0.06, 0.86, DD + 0.15), (-0.06, 0.74, DD + 0.15), (-0.12, 0.74, DD + 0.15)], (200, 30, 40), 220)
    face([(-0.27, 0.80, BD - 0.2), (-0.17, 0.80, BD - 0.2), (-0.18, 0.74, BD - 0.2), (-0.26, 0.74, BD - 0.2)], (60, 60, 64), 200)
    g0, g1, g2, g3 = P(-0.27, 1.05, BD - 0.2), P(-0.17, 1.05, BD - 0.2), P(-0.19, 0.80, BD - 0.2), P(-0.25, 0.80, BD - 0.2)
    de.polygon(k(*g0, *g1, *g2, *g3), fill=(120, 40, 10))
    bx, by = P(-0.22, 0.88, BD - 0.2); r = F * 0.022 / (BD - 0.2)
    de.ellipse(k(bx - r, by - r, bx + r, by + r), fill=(255, 120, 30))
    bx, by = P(-0.21, 0.98, BD - 0.2)
    de.ellipse(k(bx - r * 0.8, by - r * 0.8, bx + r * 0.8, by + r * 0.8), fill=(255, 150, 40))
    face([(-0.27, 1.05, BD - 0.2), (-0.17, 1.05, BD - 0.2), (-0.19, 1.10, BD - 0.2), (-0.25, 1.10, BD - 0.2)], (60, 60, 64), 200)
    # the pizza on the desk's right end: flat, turned 15 degrees as if thrown there, the lid open
    # back against the wall
    pa, pcx, pcd, phx, phd = math.radians(15), 0.6, BD - 0.24, 0.2, 0.18
    def pzp(u, v, y):                                   # u across, v deep (-1..1) in the box, turned about its middle
        x_, d_ = u * phx, v * phd
        return (pcx + x_ * math.cos(pa) - d_ * math.sin(pa), y, pcd + x_ * math.sin(pa) + d_ * math.cos(pa))
    PB = [pzp(-1, -1, 0.765), pzp(1, -1, 0.765), pzp(1, 1, 0.765), pzp(-1, 1, 0.765)]
    def lp(u, t):                                       # the lid: hinged on the box's back edge, leaning back on the wall
        x_, y_, d_ = pzp(u, 1, 0.765)
        return (x_, 0.765 + 0.35 * t, min(BD - 0.01, d_ + 0.12 * t))
    face([lp(-1, 0), lp(1, 0), lp(1, 1), lp(-1, 1)], (180, 158, 116), 230)
    face([lp(-0.88, 0.06), lp(0.88, 0.06), lp(0.88, 0.94), lp(-0.88, 0.94)], (200, 180, 136), 230)
    face(PB, (150, 128, 92), 240)
    face([pzp(-1, -1, 0.765), pzp(1, -1, 0.765), pzp(1, -1, 0.74), pzp(-1, -1, 0.74)], (190, 166, 120), 240)   # its front wall
    face([pzp(-1, -1, 0.765), pzp(-1, 1, 0.765), pzp(-1, 1, 0.74), pzp(-1, -1, 0.74)], (170, 148, 106), 240)   # its left wall
    pz = Image.new("RGBA", (400, 400), (0, 0, 0, 0)); dz = ImageDraw.Draw(pz)
    dz.pieslice((30, 30, 370, 370), 25, 290, fill=(200, 120, 50, 255))
    dz.pieslice((52, 52, 348, 348), 25, 290, fill=(240, 196, 90, 255))
    for (px_, py_) in ((150, 120), (240, 110), (110, 210), (200, 210), (150, 290), (250, 300), (280, 210)):
        if not (px_ > 230 and py_ < 200):
            dz.ellipse((px_ - 22, py_ - 22, px_ + 22, py_ + 22), fill=(170, 40, 30, 255))
    dz.ellipse((262, 120, 296, 154), fill=(120, 30, 24, 120))                                                                 # a grease spot
    decal(pz, [PB[3], PB[2], PB[1], PB[0]], 240)
    # the bookshelf over the desk (messy dorm room): two boards 0.22 m deep, books and junk; one
    # fat black book with ABRASH on its spine
    SX0, SX1, SDP = -0.38, 0.88, 0.22
    rng = np.random.default_rng(7)
    sp = ImageFont.truetype("/System/Library/Fonts/Supplemental/Impact.ttf", 200)
    for sy in (1.40, 1.76):
        face([(SX0, sy, BD - SDP), (SX1, sy, BD - SDP), (SX1, sy, BD), (SX0, sy, BD)], (128, 96, 62), 255)
        face([(SX0, sy, BD - SDP), (SX1, sy, BD - SDP), (SX1, sy - 0.03, BD - SDP), (SX0, sy - 0.03, BD - SDP)], (96, 70, 44), 255)
        for x in (SX0 + 0.06, SX1 - 0.08):
            face([(x, sy - 0.03, BD - 0.02), (x + 0.02, sy - 0.03, BD - 0.02), (x + 0.02, sy - 0.16, BD - 0.02), (x, sy - 0.16, BD - 0.02)], (70, 70, 74), 230)
    dB = BD - 0.13
    books = ((0.035, 0.24, (150, 30, 34)), (0.03, 0.22, (30, 70, 140)), (0.05, 0.26, (200, 180, 60)), (0.028, 0.2, (40, 110, 60)),
             (0.04, 0.25, (220, 220, 210)), (0.09, 0.27, "abrash"), (0.032, 0.23, (110, 50, 120)), (0.045, 0.21, (230, 120, 40)),
             (0.03, 0.24, (60, 60, 64)), (0.038, 0.22, (170, 40, 60)))
    x = SX0 + 0.12
    for wdt, ht, col in books:                                   # the lower board, left: standing books
        if col == "abrash":
            face([(x, 1.40 + ht, dB), (x + wdt, 1.40 + ht, dB), (x + wdt, 1.40, dB), (x, 1.40, dB)], (16, 16, 18), 255)
            sw, sh = 300, round(300 * ht / wdt)
            img = Image.new("RGBA", (sh, sw), (0, 0, 0, 0)); di = ImageDraw.Draw(img)
            bb = di.textbbox((0, 0), "ABRASH", font=sp)
            sc_ = min((sh * 0.86) / (bb[2] - bb[0]), (sw * 0.8) / (bb[3] - bb[1]))
            f2 = ImageFont.truetype("/System/Library/Fonts/Supplemental/Impact.ttf", max(8, int(200 * sc_)))
            bb = di.textbbox((0, 0), "ABRASH", font=f2)
            di.text(((sh - (bb[2] - bb[0])) / 2 - bb[0], (sw - (bb[3] - bb[1])) / 2 - bb[1]), "ABRASH", font=f2, fill=(236, 232, 214, 255))
            decal(img.rotate(-90, expand=True), [(x, 1.40 + ht, dB), (x + wdt, 1.40 + ht, dB), (x + wdt, 1.40, dB), (x, 1.40, dB)], 255)
        else:
            face([(x, 1.40 + ht, dB), (x + wdt, 1.40 + ht, dB), (x + wdt, 1.40, dB), (x, 1.40, dB)], col, 255)
            face([(x + wdt * 0.2, 1.40 + ht * 0.8, dB), (x + wdt * 0.8, 1.40 + ht * 0.8, dB), (x + wdt * 0.8, 1.40 + ht * 0.76, dB), (x + wdt * 0.2, 1.40 + ht * 0.76, dB)], tuple(min(255, c + 60) for c in col), 255)
        x += wdt + 0.004
    face([(x + 0.01, 1.40 + 0.2, dB), (x + 0.04, 1.40 + 0.21, dB), (x + 0.12, 1.40, dB), (x + 0.085, 1.40, dB)], (90, 140, 170), 255)   # one leaning
    x += 0.16
    for i, (th, col) in enumerate(((0.035, (180, 60, 40)), (0.03, (50, 50, 120)), (0.04, (210, 200, 170)))):               # a stack lying down
        y0 = 1.40 + sum((0.035, 0.03, 0.04)[:i])
        face([(x - 0.01 * i, y0 + th, dB), (x + 0.2 + 0.01 * i, y0 + th, dB), (x + 0.2 + 0.01 * i, y0, dB), (x - 0.01 * i, y0, dB)], col, 255)
    # the upper board: more books, a mug, a stack of floppies, a little trophy, a can pyramid
    x = SX0 + 0.05
    for i in range(9):
        wdt, ht = 0.025 + rng.random() * 0.03, 0.17 + rng.random() * 0.09
        col = tuple(int(v) for v in rng.integers(40, 220, 3))
        face([(x, 1.76 + ht, dB), (x + wdt, 1.76 + ht, dB), (x + wdt, 1.76, dB), (x, 1.76, dB)], col, 240)
        x += wdt + 0.004
    face([(x + 0.03, 1.76 + 0.1, dB), (x + 0.11, 1.76 + 0.1, dB), (x + 0.11, 1.76, dB), (x + 0.03, 1.76, dB)], (220, 216, 200), 240)    # the mug
    face([(x + 0.11, 1.76 + 0.08, dB), (x + 0.135, 1.76 + 0.08, dB), (x + 0.135, 1.76 + 0.03, dB), (x + 0.11, 1.76 + 0.03, dB)], (220, 216, 200), 240)
    x += 0.2
    for i in range(5):                                                                                                      # floppies
        face([(x, 1.76 + 0.012 * (i + 1), dB), (x + 0.09, 1.76 + 0.012 * (i + 1), dB), (x + 0.09, 1.76 + 0.012 * i, dB), (x, 1.76 + 0.012 * i, dB)], ((30, 30, 34), (180, 40, 40), (30, 60, 150), (30, 30, 34), (200, 200, 60))[i], 240)
    x += 0.14
    face([(x + 0.02, 1.76 + 0.03, dB), (x + 0.08, 1.76 + 0.03, dB), (x + 0.08, 1.76, dB), (x + 0.02, 1.76, dB)], (60, 50, 40), 240)  # the trophy
    face([(x + 0.04, 1.76 + 0.09, dB), (x + 0.06, 1.76 + 0.09, dB), (x + 0.06, 1.76 + 0.03, dB), (x + 0.04, 1.76 + 0.03, dB)], (210, 170, 50), 240)
    face([(x + 0.02, 1.76 + 0.14, dB), (x + 0.08, 1.76 + 0.14, dB), (x + 0.07, 1.76 + 0.09, dB), (x + 0.03, 1.76 + 0.09, dB)], (230, 190, 60), 240)
    x += 0.12
    for row, n in enumerate((3, 2, 1)):                                                                                     # cans
        for j in range(n):
            cx0 = x + j * 0.07 + row * 0.035
            face([(cx0, 1.76 + 0.12 * (row + 1), dB), (cx0 + 0.065, 1.76 + 0.12 * (row + 1), dB), (cx0 + 0.065, 1.76 + 0.12 * row, dB), (cx0, 1.76 + 0.12 * row, dB)], (200, 30, 40) if (row + j) % 2 == 0 else (40, 90, 190), 240)
    # the laundry heaped on the lower bunk, an open pizza box on top
    # a heap of clothes: irregular pieces, back to front, inside the heap's outline (0.5 m up at the bunk's
    # blanket to about 0.85 m under the box), each with a darker fold
    cl = np.random.default_rng(11)
    COLS = ((52, 72, 120), (70, 92, 140), (130, 130, 136), (220, 220, 214), (170, 44, 48), (30, 30, 34), (50, 110, 64),
            (200, 160, 60), (120, 90, 150), (230, 230, 230), (90, 60, 40))
    pieces = []
    for i in range(11):                                        # (a third of what it was)
        y = 0.52 + cl.random() ** 0.8 * 0.3
        hw = 0.34 * (1 - ((y - 0.5) / 0.36) ** 1.6) + 0.04
        pieces.append((3.7 - cl.random() * 0.6, -1.0 + (cl.random() * 2 - 1) * hw, y, COLS[i % len(COLS)]))
    for d_, X_, Y_, col in sorted(pieces, reverse=True):
        rx, ry = 0.09 + cl.random() * 0.08, 0.05 + cl.random() * 0.05
        n = 9
        pts = [(X_ + rx * (0.7 + 0.5 * cl.random()) * math.cos(2 * math.pi * j / n), max(0.5, Y_ + ry * (0.7 + 0.5 * cl.random()) * math.sin(2 * math.pi * j / n)), d_) for j in range(n)]
        face(pts, col, 150)
        face([(X_ - rx * 0.6, Y_ + 0.01, d_), (X_ + rx * 0.5, Y_ + 0.025, d_), (X_ + rx * 0.5, Y_ + 0.012, d_), (X_ - rx * 0.6, Y_ - 0.004, d_)], tuple(max(0, c - 40) for c in col), 140)
        if col == (230, 230, 230):                                                         # a striped one
            face([(X_ - rx * 0.5, Y_ - 0.02, d_), (X_ + rx * 0.5, Y_ - 0.02, d_), (X_ + rx * 0.5, Y_ - 0.035, d_), (X_ - rx * 0.5, Y_ - 0.035, d_)], (40, 60, 150), 150)
    # a sleeve hanging over the bunk's edge, a sock
    face([(-0.66, 0.6, 3.2), (-0.58, 0.58, 3.2), (-0.4, 0.5, 3.2), (-0.36, 0.22, 3.18), (-0.44, 0.2, 3.18), (-0.47, 0.48, 3.2)], (130, 130, 136), 150)
    # the pizza box: its base on the heap (0.4 m square, tilted), the lid open behind it, the pizza with slices gone
    # the door, open into the room, hinged at the doorway's back edge (DO1), 75 degrees: its hallway side to us,
    # the whiteboard on it: Demos and beer
    ang = math.radians(75)
    def dp_(t, y):
        return (R_ - 0.9 * t * math.sin(ang), y, DO1 - 0.9 * t * math.cos(ang))
    face([dp_(0, DH), dp_(1, DH), dp_(1, 0), dp_(0, 0)], (126, 90, 58), 180)
    face([dp_(1, DH), dp_(1.05, DH), dp_(1.05, 0), dp_(1, 0)], (90, 62, 40), 170)       # its edge
    face([dp_(0.13, 1.66), dp_(0.87, 1.66), dp_(0.87, 1.08), dp_(0.13, 1.08)], (40, 40, 44), 200)
    wb = Image.new("RGBA", (800, 600), (238, 238, 232, 255)); dw = ImageDraw.Draw(wb)
    mk = ImageFont.truetype("/System/Library/Fonts/MarkerFelt.ttc", 170, index=1)
    dw.text((400, 175), "Demos", font=mk, fill=(30, 60, 170, 255), anchor="mm")
    dw.text((400, 400), "and beer", font=mk, fill=(190, 30, 40, 255), anchor="mm")
    dw.line((150, 520, 650, 505), fill=(30, 60, 170, 255), width=14)
    decal(wb, [dp_(0.84, 1.63), dp_(0.16, 1.63), dp_(0.16, 1.11), dp_(0.84, 1.11)], 210)
    face([dp_(0.07, 0.95), dp_(0.12, 0.95), dp_(0.12, 0.9), dp_(0.07, 0.9)], (200, 180, 60), 180)   # the knob
    # the couch's back (foreground, 1.85 m off): dark red, its top 0.85 m up; four people above it, from behind
    face([(-1.6, 0.85, 1.85), (1.6, 0.85, 1.85), (1.6, -0.5, 1.85), (-1.6, -0.5, 1.85)], (70, 18, 22), 10)
    dc.polygon(q((-1.6, 0.85, 1.85), (1.6, 0.85, 1.85), (1.6, -0.5, 1.85), (-1.6, -0.5, 1.85)), fill=255)
    for X, hair, r_ in ((-0.98, "long", 0.105), (-0.48, "cap", 0.1), (0.62, "short", 0.098), (1.1, "curly", 0.118)):
        hx, hy = P(X, 1.12, 2.0)
        r = F * r_ / 2.0
        sw = F * 0.23 / 2.0
        _, sy = P(X, 0.98, 2.0)
        dp.rounded_rectangle(k(hx - sw, sy, hx + sw, sy + 200), radius=round(F * 0.1 / 2.0 * K), fill=255)
        dp.rectangle(k(hx - r * 0.42, hy + r * 0.5, hx + r * 0.42, sy + 10), fill=255)
        dp.ellipse(k(hx - r, hy - r, hx + r, hy + r), fill=255)
        if hair == "long":
            dp.rounded_rectangle(k(hx - r - 2, hy - r * 0.2, hx + r + 2, hy + r * 2.1), radius=round(r * 0.5 * K), fill=255)
        if hair == "cap":
            dp.ellipse(k(hx - r - 1, hy - r - 3, hx + r + 1, hy + r * 0.2), fill=255)
            dp.rounded_rectangle(k(hx - r * 0.5, hy + r * 0.35, hx + r * 0.5, hy + r * 0.78), radius=round(r * 0.2 * K), fill=255)
        if hair == "curly":
            for a in range(0, 360, 30):
                ax, ay = hx + math.cos(math.radians(a)) * r * 0.85, hy + math.sin(math.radians(a)) * r * 0.85
                dp.ellipse(k(ax - r * 0.42, ay - r * 0.42, ax + r * 0.42, ay + r * 0.42), fill=255)
    # someone on the top bunk, from behind like the others, sitting cross-legged facing the screen
    UX, UD, SEAT = -0.98, 3.6, 1.52
    upper = Image.new("L", (w, h), 0); du = ImageDraw.Draw(upper)    # behind the bunk's near posts, ladder, end board
    def up(*pts):
        du.polygon(q(*[(UX + x_, y_, UD) for x_, y_ in pts]), fill=255)
    up((-0.14, SEAT), (0.14, SEAT), (0.17, 1.82), (0.14, 1.89), (0.06, 1.92), (-0.06, 1.92), (-0.14, 1.89), (-0.17, 1.82))   # hunched back
    up((-0.035, 1.9), (0.035, 1.9), (0.035, 1.98), (-0.035, 1.98))                                                           # neck
    hx_, hy_ = P(UX, 2.04, UD); hr_ = F * 0.09 / UD
    du.ellipse(k(hx_ - hr_, hy_ - hr_ * 1.08, hx_ + hr_, hy_ + hr_ * 1.0), fill=255)
    # cross-legged: the knees out either side of the hips, low on the mattress
    up((-0.3, SEAT + 0.02), (-0.27, SEAT + 0.1), (-0.12, SEAT + 0.13), (0.12, SEAT + 0.13), (0.27, SEAT + 0.1), (0.3, SEAT + 0.02),
       (0.26, SEAT - 0.01), (-0.26, SEAT - 0.01))
    occ = Image.new("L", (w, h), 0); do_ = ImageDraw.Draw(occ)
    do_.polygon(q((BX0, 1.52, BN), (BX1, 1.52, BN), (BX1, 1.3, BN), (BX0, 1.3, BN)), fill=255)
    for (x, d) in ((BX1, BN), (BX0 + 0.02, BN), (BX1, BN - 0.3)):
        do_.polygon(q((x - 0.03, 1.85, d), (x + 0.03, 1.85, d), (x + 0.03, 0, d), (x - 0.03, 0, d)), fill=255)
    for i in range(4):
        y = 0.65 + i * 0.25
        do_.polygon(q((BX1 - 0.02, y + 0.03, BN - 0.02), (BX1 - 0.02, y + 0.03, BN - 0.32), (BX1 - 0.02, y, BN - 0.32), (BX1 - 0.02, y, BN - 0.02)), fill=255)
    people = Image.fromarray(np.maximum(np.asarray(people, float), np.asarray(upper, float) * (1 - np.asarray(occ, float) / 255)).astype(np.uint8))
    pm = np.asarray(people, float) / 255 * (1 - np.asarray(couch, float) / 255)
    door = Image.new("L", (w, h), 0)
    ImageDraw.Draw(door).polygon(q(dp_(0, DH), dp_(1.05, DH), dp_(1.05, 0), dp_(0, 0)), fill=255)
    through = np.asarray(opening, float) / 255 * (1 - np.asarray(couch, float) / 255) * (1 - np.asarray(door, float) / 255)
    emi = Image.fromarray(np.clip(np.asarray(emi, float) * (1 - through[..., None]) + np.asarray(hall, float) * through[..., None], 0, 255).astype(np.uint8))
    a_ = np.asarray(alb, float) * (1 - through[..., None]); alb = Image.fromarray(a_.astype(np.uint8))
    glw = Image.fromarray((np.asarray(glw, float) * (1 - through)).astype(np.uint8))
    # their rim: where the silhouette's edge faces the screen (up most)
    blur = max(1, 3 * K)
    soft = np.asarray(Image.fromarray((pm * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(blur)), float) / 255
    rim = np.clip((pm - soft) * 3.0, 0, 1) * pm
    gy = np.gradient(soft, axis=0) * blur * 4
    rim = rim * np.clip(0.25 + gy, 0, 1)
    # the glow: each surface's share times the fall-off from the screen (in the frame: near enough for a flat picture)
    yy, xx = np.mgrid[0:h, 0:w] / K
    xx = xx + bx0; yy = yy + by0
    cx, cy = (SCREEN[0] + SCREEN[2]) / 2, (SCREEN[1] + SCREEN[3]) / 2
    fall = 1.0 / (1.0 + (((xx - cx) ** 2 + ((yy - cy) / 0.8) ** 2) / 210.0 ** 2))
    gl = np.asarray(glw, float) / 255 * fall
    gl = gl * (1 - pm)
    a = np.asarray(alb, float) * (1 - pm[..., None]) + np.array([6, 6, 8]) * pm[..., None]
    sm = Image.new("L", (w, h), 0)
    ImageDraw.Draw(sm).rounded_rectangle(k(*SCREEN), radius=max(1, round(F * 0.012 / MON_D * K)), fill=255)
    out = {}
    def down(x):
        im = Image.fromarray(np.clip(x, 0, 255).astype(np.uint8))
        return np.asarray(im.resize((w // SS, h // SS), Image.BOX), float) / 255
    out["alb"] = down(a)
    out["glow"] = down(gl * 255)
    out["emi"] = down(np.asarray(emi, float) * ((1 - pm) * (1 - np.asarray(couch, float) / 255))[..., None])   # the couch in front of the tower's LEDs
    out["screen"] = down(np.asarray(sm, float))
    out["rim"] = down(rim * 255)
    out["box"], out["scale"] = box, scale
    return out


def lin(x):
    return np.where(x <= 0.04045, x / 12.92, ((x + 0.055) / 1.055) ** 2.4)


def srgb(x):
    x = np.clip(x, 0, 1)
    return np.where(x <= 0.0031308, x * 12.92, 1.055 * x ** (1 / 2.4) - 0.055)


def room_lit(L, mean, fade):
    """A layer set lit by the screen's mean colour (linear), its light faded by `fade` (0..1)."""
    light = AMB + L["glow"][..., None] * mean * GAIN
    return (lin(L["alb"]) * light + lin(L["emi"]) * 0.9 + L["rim"][..., None] * mean * RIM) * fade


def frame(Ls, rooms, picture, s, centre):
    """The frame: the room (the inset's where it covers) with the picture on the screen, through a zoom of s about
    `centre` (final-frame units)."""
    yy, xx = np.mgrid[0:H, 0:W].astype(float)
    ux = centre[0] + (xx - W / 2) / s
    uy = centre[1] + (yy - H / 2) / s
    col = np.zeros((H, W, 3)); m = np.zeros((H, W, 1))
    for L, room in zip(Ls, rooms):
        x0, y0, x1, y1 = L["box"]
        inside = (ux >= x0) & (ux < x1) & (uy >= y0) & (uy < y1)
        ah, aw = room.shape[:2]
        ax = np.clip(((ux - x0) * L["scale"]).astype(int), 0, aw - 1)
        ay = np.clip(((uy - y0) * L["scale"]).astype(int), 0, ah - 1)
        col = np.where(inside[..., None], srgb(room[ay, ax]), col)
        m = np.where(inside[..., None], L["screen"][ay, ax][..., None], m)
    ph, pw = picture.shape[:2]
    px = np.clip(((ux - SCREEN[0]) / (SCREEN[2] - SCREEN[0]) * pw).astype(int), 0, pw - 1)
    py = np.clip(((uy - SCREEN[1]) / (SCREEN[3] - SCREEN[1]) * ph).astype(int), 0, ph - 1)
    return col * (1 - m) + picture[py, px] * m


def ease(u):
    u = min(max(u, 0.0), 1.0)
    return u * u * (3 - 2 * u)


def preview(out):
    Ls = [draw_art(ART, (0, 0, W, H)), draw_art(ART_IN, INSET)]
    mx = np.memmap(os.path.join(ROOT, "segments/modex/video_hd.rgb"), np.uint8, "r").reshape(-1, 768, 1536, 3)
    pv = np.memmap(os.path.join(ROOT, "segments/polys/video.rgb"), np.uint8, "r").reshape(-1, 96, 192, 3)
    cx, cy = (SCREEN[0] + SCREEN[2]) / 2, (SCREEN[1] + SCREEN[3]) / 2
    s0 = 768 / (SCREEN[3] - SCREEN[1])          # the screen fills the frame's height: modex's 4:3 picture exactly
    sd = 1536 / (SCREEN[2] - SCREEN[0])         # Dope's 2:1 picture (letterboxed on the screen) fills the frame
    PULL, BACK, HOLD, DIVE = 66, 24, 18, 16     # pull-back from modex frame 66 over 24 frames; Dope on the monitor
                                                # in the room for 18 (Dope starts while still pulled out); the dive 16
    p = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "768x384",
                          "-r", "12", "-i", "-", "-c:v", "libx264", "-crf", "20", "-pix_fmt", "yuv420p",
                          "-movflags", "+faststart", out], stdin=subprocess.PIPE)
    def emit(img):
        a = (np.clip(img, 0, 1) * 255).astype(np.uint16)
        a = ((a[0::2, 0::2] + a[1::2, 0::2] + a[0::2, 1::2] + a[1::2, 1::2]) // 4).astype(np.uint8)
        p.stdin.write(a.tobytes())
    def zoom(u, s_far):
        # the zoom even in log scale (a steady pull), the centre moving with it
        sc = math.exp(math.log(s_far) * (1 - u))
        w_ = (sc - 1) / (s_far - 1)
        return sc, (cx * w_ + W / 2 * (1 - w_), cy * w_ + H / 2 * (1 - w_))
    stills = {}
    for f in range(40, 150):                    # modex from 40 (some context before)
        if f < PULL:
            emit(mx[f] / 255.0); continue
        pic = mx[f][:, 256:1280] / 255.0        # its 4:3 picture
        s, c = zoom(ease((f - PULL) / BACK), s0)
        mean = lin(pic[::8, ::8]).reshape(-1, 3).mean(0)
        fade = min(1.0, (f - PULL) / 8)
        img = frame(Ls, [room_lit(L, mean, fade) for L in Ls], pic, s, c)
        if f == 120: stills["room"] = img
        emit(img)
    for f in range(0, 60):                      # Dope (the cut's polys from 200), letterboxed on the screen
        src = np.repeat(np.repeat(pv[200 + f] / 255.0, 8, 0), 8, 1)
        if f >= HOLD + DIVE:
            emit(src); continue
        pic = np.zeros((1152, 1536, 3)); pic[192:960] = src     # 4:3 screen, the 2:1 picture in its middle
        u = ease((f - HOLD) / DIVE)
        s, c = zoom(1 - u, sd)
        mean = lin(src[::8, ::8]).reshape(-1, 3).mean(0) * 0.67
        emit(frame(Ls, [room_lit(L, mean, 1 - u) for L in Ls], pic, s, c))
    p.stdin.close(); p.wait()
    Image.fromarray((np.clip(stills["room"], 0, 1) * 255).astype(np.uint8)).save(out.replace(".mp4", "_still.png"))


def still(out):
    L = draw_art(ART, (0, 0, W, H))
    mx = np.memmap(os.path.join(ROOT, "segments/modex/video_hd.rgb"), np.uint8, "r").reshape(-1, 768, 1536, 3)
    pic = mx[120][:, 256:1280] / 255.0
    mean = lin(pic[::8, ::8]).reshape(-1, 3).mean(0)
    img = frame([L], [room_lit(L, mean, 1.0)], pic, 1.0, (W / 2, H / 2))
    Image.fromarray((np.clip(img, 0, 1) * 255).astype(np.uint8)).save(out)


if __name__ == "__main__":
    if sys.argv[1] == "preview":
        preview(sys.argv[2])
    if sys.argv[1] == "still":
        still(sys.argv[2])
