"""Design stage: the drive goes hardware-accelerated, genericised (no 3Dfx, Glide
or Voodoo words on screen, no logo). glide.py is the beat's spec (software drive, the VIDEO OPTIONS menu, the
blank, the drive on in the new look, the API calls 2 s, the brag); this file designs what glide.py only sketched:

* the words: the menu's renderer value, the calls over the drive and the brag, three options (WORDS);
* the look: the drive's scene rendered at size (768x384, the pixel count of 640x480 in the film's 2:1 frame, shown
  doubled to 1536x768), not glide.py's filter over the 192x96 frames: bilinear-filtered textures, glows on the
  lights, lens flares on the lamps and headlights, the purple fog, 16-bit (RGB565) 4x4 dither, our car sharp; three
  variants (LOOKS).

Floating point; a picture of the look, not a reference. The scene is the drive's own: its data cards
(segments/drive/data: the street's plan, the blocks, the window texture, the traffic, the script) and the
formulas of the old drive's reference (no longer in the repository), in world units.

    python accel.py hd LOOK OUT.mp4 [small]       the drive's second half (frames 66-183, with the jump) in look A, B or C
    python accel.py half LOOK WORDS OUT.mp4 [small]   the accelerated half as in the beat: the calls, the jump, the brag
    python accel.py beat LOOK WORDS OUT.mp4 [small]   the whole beat: software, menu, blank, the new look, calls, brag
    python accel.py stills WORDS OUT.png            the menu after APPLY, the calls and the brag, one still

`small` writes 1024x512 (nearest) instead of 1536x768.
"""
import math
import os
import subprocess
import sys
import zlib

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))

def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)

DATA = os.path.join(HERE, "../segments/drive/data")
SUP = "/System/Library/Fonts/Supplemental/"
W, H, FPS = 1536, 768, 12
RW, RH = 768, 384                                          # the accelerated drive's own pixels
SOFT = 66                                                  # the software drive's frames before the menu
MENU, BLANK = 47, 5                                        # the menu's frames (the drive goes on under it), the blank
HD0 = SOFT + MENU + BLANK                                  # the accelerated half's first frame of the drive (118)


def ints(name):
    return [list(map(int, l.split())) for l in open(os.path.join(DATA, name)) if l.strip()]


# ---- the city, from the drive's data ------------------------------------------------------------------------------

def smooth(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3 - 2 * t)


# the twisty street: its middle eased from one place across to the next every 70 units, from
# 30 on: bends left and right the whole way
BENDS = [0, -9, 5, -9, 7, -4, 8, -6, 3, -5, 6, -7, 4, -3, 0]


def road(z):
    """The street's middle across (units) at z along."""
    z = np.asarray(z, np.float64)
    out = np.zeros_like(z) + BENDS[0]
    for k in range(1, len(BENDS)):
        out = out + (BENDS[k] - BENDS[k - 1]) * smooth(30 + 70 * (k - 1), 30 + 70 * k, z)
    return out


WINDOWS = np.array([r[0] for r in ints("windows.txt")])    # 30 columns of 32 floors: 4 unlit, 5-8 lit


def _hash2(a, b):
    return ((((a * 73856093) ^ (b * 19349663)) * 2654435761) & 0xffff) / 65536.0


def _blocks(n):
    """The drive's blocks (make_data.py's), run on to n: per block left then right, the setback from the street's
    middle (eighths), height in half-unit bands, tone, window texture place."""
    out = []
    for b in range(n):
        row = []
        for side in (-1, 1):
            front = 9 + 4 * _hash2(b, side * 3)
            height = 8 + 36 * _hash2(b, side)
            floors = min(14, max(2, round((height - 1) / 3)))
            tone = 0 if _hash2(b, 7 + side) < 0.5 else 1
            row += [round(8 * front), 6 * floors + 2, tone, 32 * int(30 * _hash2(b * 977 + 13, 11 + side)) + int(16 * _hash2(b, 23 + side))]
        out.append(row)
    return out


BLOCKS = _blocks(30)
# the traffic (make_data.py's, run on: two more slow cars, more oncoming): slow cars' places at frame 0 (units),
# oncoming ones' first places, each moved on until it meets us with our car in its own lane and clear of the last
SLOW_AT = [40.0, 95.0, 150.0, 205.0, 250.0]
ONCOMING_AT = [120.0, 190.0, 230.0, 300.0, 340.0, 420.0, 480.0, 560.0, 620.0, 700.0, 760.0]
BODY_CODES = [368, 884, 888, 476, 757]
DARK_CODES = [135, 662, 666, 253, 525]


def car_x(t):
    """make_data.py's steering: our car's place across the street (units from the middle) at t seconds."""
    z = 27.0 * t
    out = max(smooth(26, 14, s0 + 15.0 * t - z) * smooth(-10, -2, s0 + 15.0 * t - z) for s0 in SLOW_AT)
    return 2.5 - out * 4 + 0.4 * math.sin(t * 1.7)


def _cars():
    meet = []
    for o in ONCOMING_AT:
        while any(car_x((o - 7) / (27 + 21) + k / 12) < 1.6 for k in range(-6, 7)) or \
                (meet and 12 * (o - 7) / (27 + 21) < meet[-1] + 10):
            o += 6
        meet.append(round(12 * (o - 7) / (27 + 21)))
    cars = [(0, round(4 * s0), k % 5) for k, s0 in enumerate(SLOW_AT)] + [(1, m, (k + 2) % 5) for k, m in enumerate(meet)]
    return [[kind, v, 20 if kind == 0 else -21, kind, BODY_CODES[c], DARK_CODES[c]] for kind, v, c in cars]


CARS = _cars()
FRAMES = HD0 + 118                                          # the whole beat's drive: 236 frames
# the script, a frame a line: the camera's place across (eighths; it follows our car a beat behind)
SCRIPT = [[round(8 * car_x(f / 12 - 0.25))] for f in range(FRAMES)]


def lerp(a, b, k):
    return np.asarray(a, np.float32) + (np.asarray(b, np.float32) - np.asarray(a, np.float32)) * k


WALL = [lerp([30, 30, 44], [64, 52, 60], 0.25), lerp([30, 30, 44], [64, 52, 60], 0.75)]
LEDGE = [lerp(w, [120, 110, 120], 0.5) for w in WALL]
UNLIT = np.array([14, 16, 26], np.float32)
LIT = [lerp([255, 214, 120], [180, 220, 255], k / 3) for k in range(4)]
ASPHALT, POOL = np.array([28, 28, 34], np.float32), np.array([120, 96, 60], np.float32)
STRIPE, EDGE = np.array([200, 190, 120], np.float32), np.array([150, 150, 150], np.float32)
WALK, BEYOND = np.array([56, 54, 58], np.float32), np.array([34, 32, 38], np.float32)
SKY_LOW, SKY_TOP = np.array([44, 26, 62], np.float32), np.array([6, 8, 22], np.float32)
LAMP, GLARE, TAIL = np.array([255, 230, 170], np.float32), np.array([255, 250, 220], np.float32), np.array([255, 50, 50], np.float32)
POST = np.array([70, 70, 76], np.float32)
BODIES = [[40, 90, 180], [220, 200, 60], [230, 230, 230], [60, 160, 90], [150, 60, 170], [176, 26, 30]]


def hash2(a, b):
    """make_data.py's hash, over arrays."""
    a = np.asarray(a, np.int64)
    b = np.asarray(b, np.int64)
    return ((((a * 73856093) ^ (b * 19349663)) * 2654435761) & 0xffff) / 65536.0


# ---- the looks ----------------------------------------------------------------------------------------------------

LOOKS = {
    # A: as the drive is, at size: the drive's haze (none to 56 units, all by 150), glows, modest flares (ghosts
    #    only down the nearest headlights' line)
    "A": dict(fog="linear", near=56, far=150, bloom=0.55, glow=1.0, flare=0.7, ghosts=1, streak=0.6),
    # B: the 1998 brochure: big flares, ghosts down every bright light's line, long streaks, more bloom
    "B": dict(fog="linear", near=56, far=150, bloom=0.8, glow=1.3, flare=1.3, ghosts=4, streak=1.2),
    # C: the fog table: thicker purple fog that starts near (exponential), the lights' halos wide in it, flares as A
    "C": dict(fog="exp", near=10, far=42, bloom=0.7, glow=1.15, flare=0.8, ghosts=1, streak=0.7),
}


# ---- the camera ---------------------------------------------------------------------------------------------------

class Cam:
    """The drive's camera at frame f: 9 quarters a frame along, its lane from the script, turned to the street's
    slope 2-14 units ahead; 2 units up; the 192x96 screen's focal length (110 pixels), horizon at row 38."""

    def __init__(self, f, h=2.0, pitch=0.0):
        self.f = f
        self.z = 9 * f / 4
        self.h = h
        self.hz = 38 + 110 * pitch                         # the horizon's row (192x96's): the view sheared up or down
        lane = SCRIPT[f][0] / 8
        self.x = road(self.z) + lane
        slope = (road(self.z + 14) - road(self.z + 2)) / 12
        th = math.atan(slope)
        self.s, self.c = math.sin(th), math.cos(th)
        self.k = RW / 192                                  # our pixels a 192x96 pixel

    def project(self, X, Y, Z):
        """(pixel x, pixel y, along, pixels a unit) of a world point."""
        dx, dz = X - self.x, Z - self.z
        a = dx * self.s + dz * self.c
        b = dx * self.c - dz * self.s
        a = max(a, 1e-3)
        u = 96 + 110 * b / a
        v = self.hz - 110 * (Y - self.h) / a
        return u * self.k - 0.5, v * self.k - 0.5, a, 110 / a * self.k


# ---- the textures -------------------------------------------------------------------------------------------------

TS = 0.25                                                  # a wall texel: a quarter unit (8 a window bay, 12 a floor)


def wall_texels(i, j, top, tone, tex, end):
    """Colour of wall texel (i along, j up) for walls of the given params (arrays)."""
    s = (i + 0.5) * TS
    y = (j + 0.5) * TS
    hgt = top * 0.5
    bay = np.floor(s / 2).astype(np.int64)
    col = (tex + 480 * end + 32 * bay) % 960
    floor = np.clip(np.floor(y / 3).astype(np.int64), 0, 31)
    m = WINDOWS[np.clip(col + floor, 0, 959)]
    wall = np.where(tone[..., None] == 0, WALL[0], WALL[1])
    ledge = np.where(tone[..., None] == 0, LEDGE[0], LEDGE[1])
    lit = np.choose(np.clip(m - 5, 0, 3)[..., None], LIT)
    # a window: lit (a little brighter up by the ceiling, some with blinds) or dark glass; its frame a texel darker
    inside_s, inside_y = s % 2, y % 3
    rim = (inside_s < 0.75) | (inside_s > 1.75) | (inside_y < 1.25) | (inside_y > 2.75)
    shade = 0.82 + 0.3 * (inside_y - 1) / 2 + 0.12 * (hash2(col * 7 + floor, i) - 0.5)
    blinds = (hash2(col + 3, floor * 5) < 0.25) & (np.floor(inside_y * 4) % 2 == 0)
    winc = np.where((m >= 5)[..., None], lit * shade[..., None] * np.where(blinds, 0.7, 1.0)[..., None],
                    UNLIT * (1 + 0.6 * (inside_y > 2.25))[..., None])
    winc = np.where(rim[..., None], winc * 0.55 + UNLIT * 0.45, winc)
    grain = 1 + 0.10 * (hash2(i * 3 + 1, j * 7 + tex) - 0.5)       # concrete
    solid = (s % 2 < 0.5) | (y % 3 < 1)
    c = np.where(solid[..., None], wall * grain[..., None], winc)
    c = np.where((y >= hgt - 1)[..., None], np.where((y >= hgt - 0.5)[..., None], ledge, wall) * grain[..., None], c)
    return np.where(end[..., None] > 0, c * 0.62, c)


SOFTWARE = False                                           # the software half: 192x96, nearest texels, no bloom or flares


def set_size(w, h, software):
    """Draw at w x h from now on (the software half at 192x96, the accelerated at 768x384)."""
    global RW, RH, YY, XX, DITH, SOFTWARE
    RW, RH, SOFTWARE = w, h, software
    YY, XX = np.mgrid[0:RH, 0:RW].astype(np.float32)
    DITH = (np.tile(B4, (RH // 4, RW // 4)) + 0.5) / 16


def bilinear_wall(s, y, top, tone, tex, end):
    """The wall's colour at (s along, y up), bilinear between the four nearest texels (the software half: the
    nearest one)."""
    if SOFTWARE:
        return wall_texels(np.floor(s / TS).astype(np.int64), np.floor(y / TS).astype(np.int64), top, tone, tex, end)
    fs, fy = s / TS - 0.5, y / TS - 0.5
    i0, j0 = np.floor(fs).astype(np.int64), np.floor(fy).astype(np.int64)
    ws, wy = (fs - i0)[..., None], (fy - j0)[..., None]
    jmax = (top * 2).astype(np.int64) - 1                   # don't sample above the roof
    j1 = np.minimum(j0 + 1, jmax)
    t = lambda i, j: wall_texels(i, j, top, tone, tex, end)
    return (t(i0, j0) * (1 - ws) * (1 - wy) + t(i0 + 1, j0) * ws * (1 - wy) +
            t(i0, j1) * (1 - ws) * wy + t(i0 + 1, j1) * ws * wy)


def band(x, a, b, w):
    """How much of a pixel w wide around x lies in [a, b] (a box-filtered stripe: the texture's mip)."""
    return np.clip((x - a) / w + 0.5, 0, 1) - np.clip((x - b) / w + 0.5, 0, 1)


# ---- the scene ----------------------------------------------------------------------------------------------------

YY, XX = np.mgrid[0:RH, 0:RW].astype(np.float32)


def scene(cam, look):
    """The walls, the street and the sky, per pixel: colour (float) and the distance along (inf for sky)."""
    u = (XX + 0.5) / cam.k
    v = (YY + 0.5) / cam.k
    dX = 110 * cam.s + (u - 96) * cam.c                       # the ray, per unit of t: along it 110
    dZ = 110 * cam.c - (u - 96) * cam.s
    dY = -(v - cam.hz)
    col = np.zeros((RH, RW, 3), np.float32)
    # the ground, which climbs and falls along the street: each column's ray marched out (a voxel-space column),
    # the nearest ground it meets for each row
    best, g = ground_march(cam, dX[0], dZ[0], v[:, 0])
    # the blocks: boxes from each face away from the street, 26 long, a 4-unit gap
    b0 = int(cam.z // 30)
    hit = np.zeros((RH, RW), bool)
    hs, hy, htop, htone, htex, hend = (np.zeros((RH, RW), np.float32) for _ in range(6))
    for b in range(max(b0 - 1, 0), min(b0 + 10, len(BLOCKS))):
        z0, z1 = 30 * b, 30 * b + 26
        mid = road(30 * b + 12)
        for side in (0, 1):
            setb, top, tone, tex = BLOCKS[b][4 * side:4 * side + 4]
            fx = mid - setb / 8 if side == 0 else mid + setb / 8
            x0, x1 = (fx - 400, fx) if side == 0 else (fx, fx + 400)
            with np.errstate(divide="ignore", invalid="ignore"):
                ta, tb = (x0 - cam.x) / dX, (x1 - cam.x) / dX
                txn, txf = np.minimum(ta, tb), np.maximum(ta, tb)
                ta, tb = (z0 - cam.z) / dZ, (z1 - cam.z) / dZ
                tzn, tzf = np.minimum(ta, tb), np.maximum(ta, tb)
            tn, tf = np.maximum(txn, tzn), np.minimum(txf, tzf)
            yh = cam.h + dY * tn
            base = hill(30 * b + 13)                           # the block stands on the street's grade at its middle
            foot = hill(cam.z + dZ * np.where(np.isfinite(tn), tn, 0))
            ok = (tn < tf) & (tn > 0) & (yh >= foot) & (yh <= base + top * 0.5) & (tn < best)
            yh = yh - base
            end = tzn > txn                                    # entered through the block's end
            px, pz = cam.x + dX * tn, cam.z + dZ * tn
            s = np.where(end, np.abs(px - fx), pz - z0)
            best = np.where(ok, tn, best)
            hit = hit | ok
            hs = np.where(ok, s, hs); hy = np.where(ok, yh, hy)
            htop = np.where(ok, top, htop); htone = np.where(ok, tone, htone)
            htex = np.where(ok, tex, htex); hend = np.where(ok, end, hend)
    along = best * 110
    # walls
    if hit.any():
        idx = np.nonzero(hit)
        col[idx] = bilinear_wall(hs[idx], hy[idx], htop[idx], htone[idx].astype(int), htex[idx].astype(np.int64),
                                 hend[idx].astype(int))
    # the street
    gr = g & ~hit
    if gr.any():
        idx = np.nonzero(gr)
        t = best[idx]
        X, Z = cam.x + dX[idx] * t, cam.z + dZ[idx] * t
        off = np.abs(X - road(Z))
        wx = np.maximum(0.06, t * 1.0 / cam.k * 1.2)            # a pixel's width across, units
        wz = np.maximum(0.12, 55 * t * t / cam.k * 1.2)
        if SOFTWARE:
            wx, wz = wx * 0 + 1e-4, wz * 0 + 1e-4             # the software half: hard edges
        c = ASPHALT * (1 + 0.12 * (hash2(np.floor(X * 6), np.floor(Z * 3)) - 0.5))[:, None] * \
            np.clip(1 - 0.0 * wx, 0, 1)[:, None]
        # the lamps' pools: under each lamp's head, 5 units out from the middle, 15 into each block
        dz = np.abs((Z % 30) - 15)
        pool = np.clip(1 - (dz / 9) ** 2, 0, 1) * np.clip(1 - ((off - 5.2) / 4.0) ** 2, 0, 1)
        c = c + (POOL - ASPHALT) * (0.7 * pool)[:, None]
        dash = band((Z % 7.5), 0, 3.75, wz)
        stripe = band(off, -0.125, 0.125, wx) * dash
        edge = band(off, 5.6875, 5.9375, wx)
        walk = band(off, 6, 9, wx)
        beyond = np.clip((off - 9) / wx + 0.5, 0, 1)
        kerb = band(off, 6, 6.15, wx)
        for k, m in ((stripe, STRIPE), (edge, EDGE), (walk, WALK * (1 + 0.5 * pool)[:, None] if True else WALK),
                     (kerb, np.array([90, 88, 92], np.float32)), (beyond, BEYOND)):
            c = c + (m - c) * k[:, None]
        col[idx] = c
    # the sky
    sky = ~g & ~hit
    k = np.clip((2 * cam.hz - 2 * v) / 76, 0, 1)[..., None]
    col = np.where(sky[..., None], SKY_LOW + (SKY_TOP - SKY_LOW) * k, col)
    along = np.where(hit | g, along, np.inf)
    return col, along


def fog_of(along, look):
    L = LOOKS[look]
    if L["fog"] == "linear":
        return np.clip((along - L["near"]) / (L["far"] - L["near"]), 0, 1)
    return 1 - np.exp(-np.maximum(along - L["near"], 0) / L["far"])


# ---- the cars -----------------------------------------------------------------------------------------------------

def car_picture(w, h, body, front=False, ss=4, lights=1.0):
    """A late-90s sports car from behind (or the front), w x h pixels, anti-aliased (drawn ss times over): RGBA
    float, and a mask of its lamps (for the glows)."""
    W2, H2 = w * ss, h * ss
    im = Image.new("RGBA", (W2, H2), (0, 0, 0, 0))
    lm = Image.new("L", (W2, H2), 0)
    d, dl = ImageDraw.Draw(im), ImageDraw.Draw(lm)
    body = tuple(int(c) for c in body)
    dark = tuple(int(c * 0.45) for c in body)
    hi = tuple(min(255, int(c * 1.35 + 30)) for c in body)
    P = lambda pts: [(x * W2, y * H2) for x, y in pts]
    tyre = (14, 14, 16, 255)
    d.rounded_rectangle(P([(0.05, 0.70), (0.21, 1.0)])[0] + P([(0.21, 1.0)])[0], radius=2 * ss, fill=tyre)
    d.rounded_rectangle(P([(0.79, 0.70)])[0] + P([(0.95, 1.0)])[0], radius=2 * ss, fill=tyre)
    d.polygon(P([(0.25, 0.02), (0.75, 0.02), (0.86, 0.34), (0.14, 0.34)]), fill=body + (255,))      # the cabin
    d.polygon(P([(0.29, 0.07), (0.71, 0.07), (0.80, 0.31), (0.20, 0.31)]),
              fill=(30, 34, 50, 255) if not front else (40, 46, 66, 255))                           # the glass
    d.polygon(P([(0.31, 0.09), (0.45, 0.09), (0.36, 0.29), (0.23, 0.29)]), fill=(70, 76, 100, 255))  # a reflection
    d.polygon(P([(0.03, 0.34), (0.97, 0.34), (0.99, 0.62), (0.01, 0.62)]), fill=body + (255,))      # the deck
    d.polygon(P([(0.03, 0.34), (0.97, 0.34), (0.975, 0.38), (0.025, 0.38)]), fill=hi + (255,))       # its edge
    d.polygon(P([(0.01, 0.62), (0.99, 0.62), (0.97, 0.82), (0.03, 0.82)]), fill=dark + (255,))      # the bumper
    if not front:
        for x0, x1 in ((0.05, 0.33), (0.67, 0.95)):                                                 # tail lamps
            d.rectangle(P([(x0, 0.42)])[0] + P([(x1, 0.55)])[0], fill=(120, 10, 14, 255))
            d.rectangle(P([(x0 + 0.01, 0.44)])[0] + P([(x1 - 0.01, 0.53)])[0], fill=(255, 60, 50, 255))
            dl.rectangle(P([(x0 + 0.01, 0.44)])[0] + P([(x1 - 0.01, 0.53)])[0], fill=int(255 * lights))
        for k in range(4):                                                                          # the grille slats
            y = 0.43 + 0.03 * k
            d.rectangle(P([(0.34, y)])[0] + P([(0.66, y + 0.012)])[0], fill=(10, 10, 12, 255))
        d.rectangle(P([(0.42, 0.63)])[0] + P([(0.58, 0.74)])[0], fill=(220, 220, 200, 255))        # the plate
        for x in (0.27, 0.73):
            d.ellipse(P([(x - 0.03, 0.74)])[0] + P([(x + 0.03, 0.82)])[0], fill=(30, 30, 32, 255))
    else:
        for x0, x1 in ((0.06, 0.30), (0.70, 0.94)):                                                 # headlamps
            d.rectangle(P([(x0, 0.42)])[0] + P([(x1, 0.55)])[0], fill=(255, 250, 225, 255))
            dl.rectangle(P([(x0, 0.42)])[0] + P([(x1, 0.55)])[0], fill=255)
        d.rectangle(P([(0.33, 0.44)])[0] + P([(0.67, 0.56)])[0], fill=(12, 12, 14, 255))
        d.rectangle(P([(0.08, 0.66)])[0] + P([(0.22, 0.72)])[0], fill=(255, 160, 40, 255))
        d.rectangle(P([(0.78, 0.66)])[0] + P([(0.92, 0.72)])[0], fill=(255, 160, 40, 255))
    im = im.resize((w, h), Image.BOX)
    lm = lm.resize((w, h), Image.BOX)
    return np.array(im, np.float32), np.array(lm, np.float32) / 255


TEX = 4                                                    # traffic textures: 4 texels a 192x96 picture's pixel


def jet_picture(w, h, f, ss=4):
    """The jet head on (make_data's shapes, finer), with its wingtips and strobe: RGBA, and its lamps."""
    W2, H2 = w * ss, h * ss
    im = Image.new("RGBA", (W2, H2), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    grey = (70, 72, 92, 255)
    cx = W2 / 2
    yw = H2 * 0.62
    d.polygon([(0, yw - H2 * 0.05), (cx, yw - H2 * 0.25), (W2, yw - H2 * 0.05), (W2, yw + H2 * 0.06),
               (cx, yw - H2 * 0.08), (0, yw + H2 * 0.06)], fill=grey)
    d.ellipse([cx - W2 * 0.09, H2 * 0.3, cx + W2 * 0.09, H2 * 0.95], fill=(84, 86, 108, 255))
    d.polygon([(cx - W2 * 0.03, H2 * 0.32), (cx, 0), (cx + W2 * 0.03, H2 * 0.32)], fill=grey)
    for x in (-0.3, 0.3):
        d.ellipse([cx + W2 * x - W2 * 0.05, yw, cx + W2 * x + W2 * 0.05, yw + H2 * 0.2], fill=(56, 58, 74, 255))
    im = im.resize((w, h), Image.BOX)
    return np.array(im, np.float32)


# ---- the hill and the jump: the street climbs to a crest, our car takes it flat out and flies -------------------
#
# The street's grade (a height for each place along it): flat, then a climb to a crest at 240 units with a lip, a
# steep fall beyond, levelling out well below where it started. Our car leaves the ground at the crest (the grade
# turns faster than it can follow), flies 1.4 s in an arc (nose up off the crest, levelling, nose down), lands on
# the downslope with sparks, bounces once, its tail wags, and it drives on down the hill. The camera follows it
# over the crest, high, the view sheared down so the street drops away and the city spreads out below. Our car is a
# mesh (prototypes/accel_models, made by accel_models.py in Blender), drawn by a small rasterizer: flat-lit
# triangles, 3 x 3 samples a pixel, a depth test against the street.

END = HD0 + 118                                            # the accelerated half: frames HD0..END-1 of the drive
D0 = 6.0                                                   # our car's middle, units ahead of the camera
PIVOT = 0.5                                                # it turns about its middle, this far up
SPEED = 27.0                                               # units a second
GRAV = 17.3                                                # units a second a second
Z_CREST = 357.0                                            # where the climb ends, at the lip (just after the calls)
F_LAUNCH = (Z_CREST - D0) / 2.25                           # the frame our car's middle reaches it
_LIP = (Z_CREST, 11.0, 0.18)                               # its place, height and grade

# the street's rises and dips the whole way: heights (units) and grades at places along, eased
# between (Hermite); the big climb to the lip at 240, the crest (the grade from 0.18 to -0.32 over 6 units), the fall
# to 300 where our car lands, then rolling on
HILLS_BEFORE = [(0, 0.0, 0.0), (50, 3.5, 0.0), (95, -1.5, 0.0), (150, 2.5, 0.0), (205, -1.0, 0.0), (250, 1.0, 0.08),
                (Z_CREST, 11.0, 0.18)]
_C = Z_CREST - 240                                          # the crest's place past the first design's
HILLS_AFTER = [(300 + _C, 10.58 - 0.32 * 54, -0.32), (345 + _C, -11.0, 0.0), (390 + _C, -7.5, 0.12), (430 + _C, -4.0, 0.0),
               (475 + _C, -8.0, 0.0), (520 + _C, -5.0, 0.08), (570 + _C, -2.0, 0.0), (1000, -2.0, 0.0)]


def hermite(points, z):
    out = np.zeros_like(z)
    for (z0, h0, g0), (z1, h1, g1) in zip(points, points[1:]):
        t = np.clip((z - z0) / (z1 - z0), 0, 1)
        L = z1 - z0
        v = (2 * t ** 3 - 3 * t ** 2 + 1) * h0 + (t ** 3 - 2 * t ** 2 + t) * L * g0 + (-2 * t ** 3 + 3 * t ** 2) * h1 + (t ** 3 - t ** 2) * L * g1
        out = np.where((z >= z0) & (z < z1), v, out)
    return out


def hill(z):
    """The street's height (units) at z along."""
    z = np.asarray(z, np.float64)
    d = np.clip(z - Z_CREST, 0, 6)
    crest = 11 + 0.18 * d - 0.5 / 12 * d * d
    v = np.where(z < Z_CREST, hermite(HILLS_BEFORE, z), np.where(z < Z_CREST + 6, crest,
                 np.where(z < Z_CREST + 60, 10.58 - 0.32 * (z - Z_CREST - 6), hermite(HILLS_AFTER, z))))
    return v


def grade(z):
    return (hill(z + 0.25) - hill(z - 0.25)) / 0.5


def ground_march(cam, dX, dZ, v):
    """For each pixel the ray's t (110 along a unit of t) where it first meets the ground, and whether it does: each
    column's ray stepped out, each step's ground point's row, the rows below the highest so far filled."""
    a = np.concatenate([np.arange(0.5, 20, 0.25), np.arange(20, 80, 0.5), np.arange(80, 330, 1.5)])   # along, units
    t = a / 110
    Z = cam.z + dZ[None, :] * t[:, None]
    row = cam.hz - 110 * (hill(Z) - cam.h) / a[:, None]    # the ground point's row, 192x96's, (steps, columns)
    low = np.minimum.accumulate(row, axis=0)               # the highest seen so far (rows count down)
    best = np.full((len(v), len(dX)), np.inf, np.float32)
    for x in range(len(dX)):
        r = -low[:, x]                                     # rising
        k = np.searchsorted(r, -v, side="left")            # the first step whose ground is at or above the row
        ok = k < len(a)
        kk = np.minimum(k, len(a) - 1)
        k0 = np.maximum(kk - 1, 0)
        r0, r1 = low[k0, x], low[kk, x]
        w = np.where(r0 > r1, (r0 - v) / np.maximum(r0 - r1, 1e-6), 0)
        tt = t[k0] + (t[kk] - t[k0]) * np.clip(w, 0, 1)
        tt = np.where(k == 0, t[0], tt)
        best[:, x] = np.where(ok, tt, np.inf)
    return best, np.isfinite(best)


def slope(z):
    return (road(z + 0.5) - road(z - 0.5))


def flight():
    """When our car comes down (frame), solving the arc against the fall."""
    f = F_LAUNCH
    while True:
        f += 0.01
        t = (f - F_LAUNCH) / 12
        if _LIP[1] + _LIP[2] * SPEED * t - GRAV / 2 * t * t <= float(hill(9 * f / 4 + ahead(f))) and f > F_LAUNCH + 1:
            return f




def load_obj(name):
    """Triangles (n, 3, 3: across, along, up), colours (n, 3) and material names (n), from the .obj and its .mtl."""
    base = os.path.join(HERE, "accel_models", name)
    mats, cur = {}, None
    for l in open(base + ".mtl"):
        p = l.split()
        if p and p[0] == "newmtl":
            cur = p[1]
        elif p and p[0] == "Kd":
            mats[cur] = np.array([float(v) ** (1 / 2.2) * 255 for v in p[1:4]], np.float32)
    vs, tris, cols, emit, cur = [], [], [], [], None
    for l in open(base + ".obj"):
        p = l.split()
        if not p:
            continue
        if p[0] == "v":
            vs.append([float(v) for v in p[1:4]])
        elif p[0] == "usemtl":
            cur = p[1]
        elif p[0] == "f":
            idx = [int(q.split("/")[0]) - 1 for q in p[1:]]
            for k in range(1, len(idx) - 1):
                tris.append([vs[idx[0]], vs[idx[k]], vs[idx[k + 1]]])
                cols.append(mats[cur])
                emit.append(cur)
    return np.array(tris, np.float32), np.array(cols, np.float32), np.array(emit)


CAR = None


def model():
    global CAR
    if CAR is None:
        CAR = load_obj("car")
    return CAR


def rot_z(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])


def rot_x(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])


def rot_y(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])


def ahead(f):
    """How far our car is ahead of the camera: a little further in the air, the camera catching up after."""
    if F_LAUNCH <= f < F_LAUNCH + 30:
        return D0 + 1.8 * math.sin(math.pi * (f - F_LAUNCH) / 30)
    return D0


F_LAND = flight()
F_BOUNCE = F_LAND + 6


def car_height(f):
    """Our car's wheels' height (world) and its arc's grade: on the street, in the air, the bounce."""
    z = 9 * f / 4 + ahead(f)
    g = float(hill(z))
    if F_LAUNCH <= f < F_LAND:
        t = (f - F_LAUNCH) / 12
        return _LIP[1] + _LIP[2] * SPEED * t - GRAV / 2 * t * t, (_LIP[2] * SPEED - GRAV * t) / SPEED
    if F_LAND <= f < F_BOUNCE:
        u = (f - F_LAND) / (F_BOUNCE - F_LAND)
        return g + 0.5 * 4 * u * (1 - u), float(grade(z))
    return g, float(grade(z))


def cam_place(f):
    """The camera's height and its view's tilt: 2 up over the street, or following our car up and over the crest a
    beat behind; tilted with our car's grade (sheared: the horizon moved), down to look over the drop."""
    z = 9 * f / 4
    lift = 1.4 * math.sin(math.pi * (f - F_LAUNCH) / 34) if F_LAUNCH <= f < F_LAUNCH + 34 else 0.0
    h = max(float(hill(z)) + 2.0, car_height(f - 1.5)[0] + 1.7 + lift)
    tilt = sum(car_height(f - k)[1] for k in (0, 1, 2)) / 3
    return h, float(np.clip(0.55 * tilt, -0.26, 0.16))


def car_pose(f):
    """Our car's middle (across, along, up) and its turn (a 3 x 3 matrix: across, along, up)."""
    t = f / 12
    y, gr = car_height(f)
    pitch = math.atan(gr)
    yaw = 0.0
    roll = 0.0
    if F_LAUNCH <= f < F_LAND:
        u = (f - F_LAUNCH) / (F_LAND - F_LAUNCH)
        pitch = math.atan(_LIP[2]) * (1 - u) + math.atan(gr) * 0.45 * u     # nose up off the crest, down to land
        roll = math.radians(4) * math.sin(math.pi * u)
    elif f >= F_LAND:
        g = f - F_LAND
        pitch += math.radians(-6) * math.exp(-g / 3) * math.cos(g * 1.3)
        yaw = math.radians(14) * math.exp(-g / 6) * math.sin(g * 1.1)
    z = 9 * f / 4 + ahead(f)
    x = road(z) + car_x(t)
    steer = (car_x(t + 0.05) - car_x(t - 0.05)) / (SPEED * 0.1)
    heading = math.atan(slope(z)) + math.atan(steer) + yaw
    bob = 0.012 * math.sin(t * 9) if not F_LAUNCH <= f < F_BOUNCE else 0.0
    rot = rot_z(-heading) @ rot_x(pitch) @ rot_y(roll)
    return np.array([x, z, y + PIVOT + bob]), rot


def car_point(pos, rot, local):
    return pos + rot @ (np.asarray(local, np.float64) - np.array([0, 0, PIVOT]))


def car_tris(pos, rot):
    tris, cols, emit = model()
    w = (tris.reshape(-1, 3) - np.array([0, 0, PIVOT])) @ rot.T + pos
    return w.reshape(-1, 3, 3), cols, np.char.startswith(emit.astype(str), "emit_")


LIGHT = np.array([0.35, 1.0, 0.6]) / np.linalg.norm([0.35, 1.0, 0.6])   # across, up, along: from above, ahead


def draw_meshes(col, along, cam, items, ss=3):
    """Rasterize lit triangles over the frame where nearer than what is there; 3 x 3 samples a pixel."""
    P, C, E = [], [], []
    for tris, cols, emit in items:
        P.append(tris); C.append(cols); E.append(emit)
    tris, cols, emit = np.concatenate(P), np.concatenate(C), np.concatenate(E)
    X, Z, Y = tris[..., 0], tris[..., 1], tris[..., 2]       # across, along, up
    dx, dz = X - cam.x, Z - cam.z
    a = dx * cam.s + dz * cam.c
    b = dx * cam.c - dz * cam.s
    keep = (a > 0.3).all(1)
    if not keep.any():
        return
    a, b, Y, tris, cols, emit = a[keep], b[keep], Y[keep], tris[keep], cols[keep], emit[keep]
    sx = (96 + 110 * b / a) * cam.k - 0.5
    sy = (cam.hz - 110 * (Y - cam.h) / a) * cam.k - 0.5
    w = np.stack([tris[..., 0], tris[..., 2], tris[..., 1]], -1)     # across, up, along
    n = np.cross(w[:, 1] - w[:, 0], w[:, 2] - w[:, 0])
    n /= np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)
    view = w.mean(1) - np.array([cam.x, cam.h, cam.z])
    view /= np.linalg.norm(view, axis=1, keepdims=True)
    n *= np.where((n * view).sum(1) > 0, -1, 1)[:, None]
    lam = np.clip((n * LIGHT).sum(1), 0, 1)
    half = LIGHT - view
    half /= np.linalg.norm(half, axis=1, keepdims=True)
    spec = np.clip((n * half).sum(1), 0, 1) ** 24
    lit = cols * (0.30 + 0.75 * lam)[:, None] + 70 * spec[:, None]
    lit = np.where(emit[:, None], cols, lit)
    x0, x1 = max(0, int(sx.min()) - 1), min(RW, int(sx.max()) + 2)
    y0, y1 = max(0, int(sy.min()) - 1), min(RH, int(sy.max()) + 2)
    if x0 >= x1 or y0 >= y1:
        return
    hw, hh = (x1 - x0) * ss, (y1 - y0) * ss
    depth = np.full((hh, hw), np.inf, np.float32)
    rgb = np.zeros((hh, hw, 3), np.float32)
    offs = (np.arange(ss) + 0.5) / ss - 0.5
    for t in np.argsort(-a.mean(1)):
        px, py, pa = sx[t], sy[t], a[t]
        bx0, bx1 = max(x0, int(px.min())), min(x1, int(px.max()) + 2)
        by0, by1 = max(y0, int(py.min())), min(y1, int(py.max()) + 2)
        if bx0 >= bx1 or by0 >= by1:
            continue
        gx = (np.arange(bx0, bx1)[:, None] + offs[None, :]).ravel()
        gy = (np.arange(by0, by1)[:, None] + offs[None, :]).ravel()
        GX, GY = np.meshgrid(gx, gy)
        area = (px[1] - px[0]) * (py[2] - py[0]) - (px[2] - px[0]) * (py[1] - py[0])
        if abs(area) < 1e-9:
            continue
        w0 = ((px[1] - GX) * (py[2] - GY) - (px[2] - GX) * (py[1] - GY)) / area
        w1 = ((px[2] - GX) * (py[0] - GY) - (px[0] - GX) * (py[2] - GY)) / area
        w2 = 1 - w0 - w1
        inside = (w0 >= 0) & (w1 >= 0) & (w2 >= 0)
        d = w0 * pa[0] + w1 * pa[1] + w2 * pa[2]
        ys, xs = (by0 - y0) * ss, (bx0 - x0) * ss
        reg = depth[ys:ys + GY.shape[0], xs:xs + GX.shape[1]]
        on = inside & (d < reg)
        reg[on] = d[on]
        rgb[ys:ys + GY.shape[0], xs:xs + GX.shape[1]][on] = lit[t]
    hit = np.isfinite(depth)
    cov = hit.reshape(y1 - y0, ss, x1 - x0, ss).mean((1, 3))
    c = (rgb * hit[..., None]).reshape(y1 - y0, ss, x1 - x0, ss, 3).sum((1, 3)) / np.maximum(cov * ss * ss, 1)[..., None]
    dmin = np.where(hit, depth, np.inf).reshape(y1 - y0, ss, x1 - x0, ss).min((1, 3))
    reg = col[y0:y1, x0:x1]
    vis = (along[y0:y1, x0:x1] > dmin) * cov
    col[y0:y1, x0:x1] = reg + (c - reg) * vis[..., None]
    along[y0:y1, x0:x1] = np.where(vis > 0.5, dmin, along[y0:y1, x0:x1])


def shadow(col, cam, pos, rot):
    """Our car's shadow on the street under it, fainter and smaller the higher it is."""
    g = float(hill(pos[1]))
    h = pos[2] - PIVOT - g
    k = 0.6 * float(np.clip(1 - h / 4, 0, 1))
    if k <= 0:
        return
    fwd = rot @ np.array([0, 1.0, 0])
    fwd = np.array([fwd[0], fwd[1]]) / max(1e-6, math.hypot(fwd[0], fwd[1]))
    pts = [cam.project(pos[0] + fwd[0] * e, float(hill(pos[1] + fwd[1] * e)), pos[1] + fwd[1] * e) for e in (-2.3, 2.3)]
    cx, cy = (pts[0][0] + pts[1][0]) / 2, (pts[0][1] + pts[1][1]) / 2
    _, _, a, sc = cam.project(pos[0], g, pos[1])
    rx, ry = 1.15 * sc * (1 - 0.2 * h / 4), max(2.0, abs(pts[0][1] - pts[1][1]) / 2)
    x0, x1 = max(0, int(cx - rx)), min(RW, int(cx + rx) + 1)
    y0, y1 = max(0, int(cy - ry)), min(RH, int(cy + ry) + 1)
    if x0 >= x1 or y0 >= y1:
        return
    yy, xx = np.mgrid[y0:y1, x0:x1].astype(np.float32)
    e = ((xx - cx) / rx) ** 2 + ((yy - cy) / ry) ** 2
    col[y0:y1, x0:x1] *= (1 - k * np.clip(1.25 - e, 0, 1))[..., None]


def spark_set(f0, n, seed, where):
    rng = np.random.default_rng(seed)
    return [(f0, where, np.array([rng.uniform(-6, 6), SPEED * rng.uniform(0.0, 0.45), rng.uniform(1.0, 7)]), rng.uniform(0.6, 1.0))
            for _ in range(n)]


SPARKS = None


def sparks(col, along, lights, cam, f):
    """Sparks where our car comes down: hot streaks thrown up off the street, falling, gone in 0.6 s."""
    global SPARKS
    if SPARKS is None:
        SPARKS = []
        for f0, n, seed, ends in ((F_LAND, 120, 1, (-1.7, 1.7)), (F_BOUNCE, 40, 2, (-1.7,))):
            pos, rot = car_pose(f0)
            for e in ends:
                p = car_point(pos, rot, [0, e, 0.05])
                SPARKS += spark_set(f0, n // len(ends), seed + 10 * len(SPARKS), np.array([p[0], p[1], 0.05]))
    glows = []
    for f0, p0, v, heat in SPARKS:
        age = (f - f0) / 12
        if not 0 <= age < 0.6:
            continue
        pts = []
        for tt in (age, max(0.0, age - 0.06)):
            p = p0 + v * tt + np.array([0, 0, -10 * tt * tt])
            p[2] = abs(p[2]) + float(hill(p[1]))
            x, y, a, _ = cam.project(p[0], p[2], p[1])
            pts.append((x, y, a))
        if pts[0][2] < 1:
            continue
        bright = heat * (1 - age / 0.6) ** 1.5
        (xa, ya, _), (xb, yb, _) = pts
        steps = int(max(abs(xa - xb), abs(ya - yb))) + 2
        for j in range(steps):
            u = j / (steps - 1)
            x, y = xa + (xb - xa) * u, ya + (yb - ya) * u
            for ox in (0, 1):
                xi, yi = int(x) + ox, int(y)
                if 0 <= xi < RW and 0 <= yi < RH and along[yi, xi] > pts[0][2] - 0.5:
                    col[yi, xi] += np.array([255, 200, 110.]) * bright * (1 - 0.6 * u) * (1.6 if ox == 0 else 0.7)
        glows.append((bright, xa, ya))
    glows.sort(key=lambda g: -g[0])
    for bright, x, y in glows[:10]:
        if not (0 <= x < RW and 0 <= y < RH) or along[int(y), int(x)] < 30:
            continue
        lights.append((x, y, 300, np.array([255, 170, 60.]), 0.6 * bright, "tip"))


# ---- the frame ----------------------------------------------------------------------------------------------------

def blur(img, r):
    """A gaussian-ish blur: three box blurs of radius r (rows then columns), edges clamped."""
    out = img
    for axis in (0, 1):
        for _ in range(3):
            n = out.shape[axis]
            pad = [(0, 0)] * out.ndim
            pad[axis] = (r + 1, r)
            p = np.pad(out, pad, mode="edge")
            c = np.cumsum(p, axis=axis, dtype=np.float32)
            hi = np.take(c, np.arange(2 * r + 1, 2 * r + 1 + n), axis=axis)
            lo = np.take(c, np.arange(0, n), axis=axis)
            out = (hi - lo) / (2 * r + 1)
    return out


def splat(img, x, y, colour, amount, radius, aspect=1.0):
    """Add a soft round light (a gaussian core and a long 1/r^2 halo) centred at (x, y), within its box."""
    R = int(radius * 6) + 2
    x0, x1 = max(0, int(x) - int(R * aspect)), min(img.shape[1], int(x) + int(R * aspect) + 1)
    y0, y1 = max(0, int(y) - R), min(img.shape[0], int(y) + R + 1)
    if x0 >= x1 or y0 >= y1:
        return
    yy, xx = np.mgrid[y0:y1, x0:x1].astype(np.float32)
    r2 = ((xx - x) / aspect) ** 2 + (yy - y) ** 2
    k = np.exp(-r2 / (2 * (radius * 0.35) ** 2)) + 0.35 / (1 + r2 / radius ** 2)
    img[y0:y1, x0:x1] += (k * amount)[..., None] * colour / 255 * 200


def flare(img, x, y, colour, s, look, ghosts):
    """A late-90s lens flare on a light: a four-point star, a horizontal streak, and (ghosts) a chain of coloured
    discs and rings down the line through the screen's middle."""
    L = LOOKS[look]
    R = 40 * s
    x0, x1 = max(0, int(x - 8 * R)), min(RW, int(x + 8 * R) + 1)
    y0, y1 = max(0, int(y - R)), min(RH, int(y + R) + 1)
    if x0 < x1 and y0 < y1:
        yy, xx = np.mgrid[y0:y1, x0:x1].astype(np.float32)
        ax, ay = np.abs(xx - x), np.abs(yy - y)
        streak = np.exp(-ay ** 2 / (2 * (0.9 + 0.5 * s) ** 2)) * np.exp(-ax / (90 * s * L["streak"] + 1))
        star = np.exp(-ay / (0.6 + 0.3 * s)) * np.exp(-ax / (14 * s + 1)) + \
            np.exp(-ax / (0.6 + 0.3 * s)) * np.exp(-ay / (14 * s + 1))
        img[y0:y1, x0:x1] += (streak * 230 * L["streak"] * s)[..., None] * np.array([0.55, 0.7, 1.0]) + \
            (star * 120 * s)[..., None] * colour / 255
    if not ghosts:
        return
    cx, cy = RW / 2, RH / 2
    for t, rad, c, ring in ((0.45, 10, (0.5, 1.0, 0.6), False), (0.8, 5, (1.0, 0.7, 0.3), False),
                            (1.25, 22, (0.4, 0.6, 1.0), True), (1.6, 8, (1.0, 0.5, 0.8), False),
                            (2.1, 34, (0.5, 0.8, 1.0), True)):
        gx, gy = x + (cx - x) * t, y + (cy - y) * t
        rr = rad * (0.6 + 0.6 * s)
        bx0, bx1 = max(0, int(gx - rr - 3)), min(RW, int(gx + rr + 4))
        by0, by1 = max(0, int(gy - rr - 3)), min(RH, int(gy + rr + 4))
        if bx0 >= bx1 or by0 >= by1:
            continue
        yy, xx = np.mgrid[by0:by1, bx0:bx1].astype(np.float32)
        d = np.sqrt((xx - gx) ** 2 + (yy - gy) ** 2)
        k = np.clip(1 - np.abs(d - rr * 0.85) / (rr * 0.18 + 1), 0, 1) if ring else np.clip((rr - d) / 1.5, 0, 1) * 0.6
        img[by0:by1, bx0:bx1] += (k * 34 * s * L["flare"])[..., None] * np.array(c)


B4 = np.array([0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5], np.float32).reshape(4, 4)
DITH = (np.tile(B4, (RH // 4, RW // 4)) + 0.5) / 16


def rgb565(img):
    """16-bit: five bits of red, six of green, five of blue, through the 4x4 ordered dither."""
    q = np.array([255 / 31, 255 / 63, 255 / 31], np.float32)
    return np.clip(np.floor(np.clip(img, 0, 255) / q + DITH[..., None]) * q, 0, 255)


def frame(f, look):
    """The drive's frame f in the accelerated look, RW x RH, float RGB before the 16-bit dither."""
    L = LOOKS[look]
    cam = Cam(f, *cam_place(f))
    col, along = scene(cam, look)
    lights = []                                           # (x, y, along, colour, strength, kind)
    # the lamps: a post 5 units up each side, 15 units into each block, its head 0.8 out over the street
    b0 = int(cam.z // 30)
    things = []
    for b in range(max(b0 - 1, 0), b0 + 8):
        mid = road(30 * b + 15)
        for side in (-1, 1):
            things.append(("lamp", mid + side * 6.625, 30 * b + 15, side))
    # the traffic
    for kind, v, lane, art, body, dark in CARS:
        ahead = (v - 4 * cam.f) if kind == 0 else (28 + 16 * (v - cam.f) if abs(v - cam.f) <= 50 else 9999)
        if ahead < 0 or ahead > 900:
            continue
        Z = cam.z + ahead / 4
        things.append(("car", road(Z) + lane / 8, Z, (kind, CARS.index([kind, v, lane, art, body, dark]))))
    # far to near
    def dist(t):
        return (t[1] - cam.x) * cam.s + (t[2] - cam.z) * cam.c
    things.sort(key=lambda t: -dist(t))
    fogk = lambda a: fog_of(np.float32(a), look)
    for kind, X, Z, extra in things:
        a = dist((kind, X, Z))
        if a < 3 or a > 230:
            continue
        if kind == "lamp":
            side = extra
            G = float(hill(Z))
            px, pfoot, _, sc = cam.project(X, G, Z)
            _, ptop, _, _ = cam.project(X, G + 5, Z)
            hw = max(0.6, 0.13 * sc)
            x0, x1 = int(max(0, px - hw - 1)), int(min(RW, px + hw + 2))
            y0, y1 = int(max(0, ptop)), int(min(RH, pfoot + 1))
            fk = fogk(a)
            if x0 < x1 and y0 < y1:
                xx = np.arange(x0, x1, dtype=np.float32)
                cov = np.clip(hw + 0.5 - np.abs(xx - px), 0, 1)
                shade = 0.7 + 0.6 * np.clip((xx - px) / hw * -side, -0.5, 0.5)
                c = POST * shade[:, None]
                c = c + (SKY_LOW - c) * fk
                reg = col[y0:y1, x0:x1]
                vis = (along[y0:y1, x0:x1] > a) * cov[None, :]
                col[y0:y1, x0:x1] = reg + (c[None] - reg) * vis[..., None]
                along[y0:y1, x0:x1] = np.where(vis > 0.5, a, along[y0:y1, x0:x1])
            # the arm and the head
            hx, hy, _, _ = cam.project(X - side * 0.8, G + 5, Z)
            ax0, ax1 = int(max(0, min(px, hx) - 1)), int(min(RW, max(px, hx) + 2))
            ah = max(1, int(0.12 * sc))
            if ax0 < ax1 and 0 <= int(hy) < RH:
                seg = col[int(hy) - ah:int(hy) + 1, ax0:ax1]
                vis = along[int(hy) - ah:int(hy) + 1, ax0:ax1] > a
                seg[vis] = (POST + (SKY_LOW - POST) * fk) * 0.8
            if 0 <= hx < RW and 0 <= hy < RH and along[int(hy), int(hx)] >= a - 0.5:
                lights.append((hx, hy + 0.06 * sc, a, LAMP, (1 - fk) * min(1.0, 0.25 + sc / 40), "lamp"))
        else:
            ck, ci = extra
            _, _, lane, art, body, dark = CARS[ci]
            px, py, _, sc = cam.project(X, float(hill(Z)), Z)
            w, h = 2.25 * sc, 0.875 * sc
            if w < 1:
                continue
            x0, y0 = px - w / 2, py - h
            bc = np.array(BODIES[[368, 884, 888, 476, 757, 722].index(body)] if body in (368, 884, 888, 476, 757, 722) else BODIES[0])
            tw, th = int(round(w)) + 1, int(round(h)) + 1
            pic, lamps = car_picture(36 * TEX, 14 * TEX, bc, front=art == 1)
            # the texture drawn at 144x56, sampled bilinear onto its box (the card's filter)
            spr = np.array(Image.fromarray(pic.astype(np.uint8), "RGBA").resize((tw, th), Image.BILINEAR), np.float32)
            ix0, iy0 = int(math.floor(x0)), int(math.floor(y0))
            sx0, sy0 = max(0, -ix0), max(0, -iy0)
            ex, ey = min(tw, RW - ix0), min(th, RH - iy0)
            if sx0 < ex and sy0 < ey:
                reg = col[iy0 + sy0:iy0 + ey, ix0 + sx0:ix0 + ex]
                rgb, al = spr[sy0:ey, sx0:ex, :3], spr[sy0:ey, sx0:ex, 3:] / 255
                fk = fogk(a)
                rgb = rgb + (SKY_LOW - rgb) * fk
                vis = (along[iy0 + sy0:iy0 + ey, ix0 + sx0:ix0 + ex] > a)[..., None]
                col[iy0 + sy0:iy0 + ey, ix0 + sx0:ix0 + ex] = reg + (rgb - reg) * al * vis
                along[iy0 + sy0:iy0 + ey, ix0 + sx0:ix0 + ex] = np.where((al[..., 0] > 0.5) & vis[..., 0], a,
                                                                        along[iy0 + sy0:iy0 + ey, ix0 + sx0:ix0 + ex])
            fk = float(fogk(a))
            for lx in (0.18, 0.82):
                lxp, lyp = x0 + lx * w, y0 + 0.485 * h
                if art == 1:
                    lights.append((lxp, lyp, a, GLARE, (1 - fk) * min(1.3, 0.35 + sc / 25), "head"))
                else:
                    lights.append((lxp, lyp, a, TAIL, (1 - fk) * min(0.6, 0.15 + sc / 60), "tail"))
    # the jet, down the strip of sky (behind everything but the sky)
    if 104 <= f < 140:
        k = f - 103
        size = (4 + k * k / 36) / 4 * cam.k                  # half its span, our pixels
        jx, jy = (368 + 10 * k / 9) / 4 * cam.k, (120 - 10 * k * k / 81) / 4 * cam.k
        top = jy - 9 * size / 16
        jw, jh = max(2, int(2 * size)), max(1, int(7 * size / 8))
        pic = jet_picture(jw, jh, f)
        ix0, iy0 = int(jx - size), int(top)
        for yy in range(jh):
            for_y = iy0 + yy
            if not 0 <= for_y < RH:
                continue
            xs = slice(max(0, ix0), min(RW, ix0 + jw))
            seg = pic[yy, max(0, -ix0):max(0, -ix0) + (xs.stop - xs.start)]
            sky = np.isinf(along[for_y, xs])[:, None]
            al = seg[:, 3:] / 255 * sky
            col[for_y, xs] = col[for_y, xs] + (seg[:, :3] - col[for_y, xs]) * al
        lit = f % 4 < 2
        if lit:
            lights.append((ix0 + 0.02 * jw, top + 0.6 * jh, 300, np.array([255, 50, 50.]), 0.5, "tip"))
            lights.append((ix0 + 0.98 * jw, top + 0.6 * jh, 300, np.array([60, 255, 90.]), 0.5, "tip"))
        if f % 6 == 0:
            lights.append((ix0 + jw / 2, top + 0.5 * jh, 300, np.array([255, 255, 255.]), 0.7, "tip"))
    # the haze over everything at a distance
    fk = fog_of(along, look)
    col = col + (SKY_LOW - col) * fk[..., None]
    # our car's shadow on the street, then our car, a mesh in the street (sharp, not hazed: it is near)
    pos, rot = car_pose(f)
    shadow(col, cam, pos, rot)
    draw_meshes(col, along, cam, [car_tris(pos, rot)], 1 if SOFTWARE else 3)
    # the lit tail lamps glow too, while they face us
    back = rot @ np.array([0.0, -1.0, 0.0])
    for lx in (-0.70, 0.70):
        p = car_point(pos, rot, [lx, -2.14, 0.495])
        to_cam = np.array([cam.x - p[0], cam.h - p[2], cam.z - p[1]])
        facing = (back[0] * to_cam[0] + back[2] * to_cam[1] + back[1] * to_cam[2]) / np.linalg.norm(to_cam)
        x, y, a, sc = cam.project(p[0], p[2], p[1])
        if facing > 0.05 and a > 0.5:
            lights.append((x, y, a, TAIL, 0.55 * min(1.0, facing * 2), "ourtail"))
    sparks(col, along, lights, cam, f)
    # the lights: a glow each (fog widens them in look C), then the bloom of everything bright
    glow = np.zeros_like(col)
    for x, y, a, c, s, kind in lights:
        if s <= 0.02:
            continue
        sc = 110 / max(a, 3) * cam.k
        if kind == "lamp":
            splat(glow, x, y, c, 0.9 * s * L["glow"], max(3.0, 0.9 * sc) * (1.4 if L["fog"] == "exp" else 1))
        elif kind == "head":
            splat(glow, x, y, c, 0.7 * s * L["glow"], max(3.0, 0.6 * sc) * (1.4 if L["fog"] == "exp" else 1), 1.4)
        elif kind == "tail":
            splat(glow, x, y, c, 0.6 * s * L["glow"], max(2.0, 0.35 * sc), 1.6)
        elif kind == "ourtail":
            splat(glow, x, y, c, 0.35 * s / 0.55 * L["glow"], 18 * min(2.0, 4.0 / max(a, 1.0)) * cam.k / 4, 1.8)
        else:
            splat(glow, x, y, c, s, 3)
    col = col + glow
    if SOFTWARE:
        return col
    bright = np.clip(col - 150, 0, None)
    small = bright[::4, ::4]
    bl = blur(small, 3) * 0.6 + blur(small, 9) * 0.4
    col = col + np.repeat(np.repeat(bl, 4, 0), 4, 1)[:RH, :RW] * L["bloom"] * 1.5
    # the flares, on the lamps and the headlights; ghosts down the strongest
    fl = [(s, x, y, c, kind) for x, y, a, c, s, kind in lights if kind in ("lamp", "head") and s > 0.12]
    fl.sort(key=lambda t: -t[0])
    for i, (s, x, y, c, kind) in enumerate(fl):
        ghosts = i < L["ghosts"] and (kind == "head" or L["ghosts"] > 1) and s > 0.45
        flare(col, x, y, c, min(1.2, s) * L["flare"] * (1.0 if kind == "head" else 0.6), look, ghosts)
    return col


def hd_frame(f, look, cache=True):
    """A frame of the accelerated drive at W x H: the 16-bit frame, each pixel doubled. Cached as .npy."""
    p = _work(f"pkg8/cache_v2/{look}{zlib.crc32(repr(sorted(LOOKS[look].items())).encode()) % 10000:04d}_{f:03d}.npy")
    if cache and os.path.exists(p):
        return np.load(p)
    img = rgb565(frame(f, look)).astype(np.uint8)
    if cache:
        os.makedirs(os.path.dirname(p), exist_ok=True)
        np.save(p, img)
    return img


LEVELS = [[0, 12, 26, 38, 52, 70, 100, 150, 205, 255], [0, 12, 26, 36, 48, 62, 90, 140, 205, 255], [0, 16, 28, 40, 54, 70, 100, 150, 205, 255]]


def software(img):
    """The software renderer's colours: each channel one of the drive's ten levels, between two through the 4 x 4
    ordered dither."""
    out = np.zeros_like(img)
    for c in range(3):
        lv = np.array(LEVELS[c], np.float32)
        v = np.clip(img[..., c], 0, 255)
        i = np.clip(np.searchsorted(lv, v, side="right") - 1, 0, 8)
        lo, hi = lv[i], lv[i + 1]
        frac = (v - lo) / (hi - lo)
        out[..., c] = np.where(frac > DITH, hi, lo)
    return out


def soft_frame(f, cache=True):
    """A frame of the software half: the street at 192x96 as the software renderer drew it, its own colours."""
    p = _work(f"pkg8/cache_v2_soft/{f:03d}.npy")
    if cache and os.path.exists(p):
        return np.load(p)
    set_size(192, 96, True)
    try:
        img = software(frame(f, "A")).astype(np.uint8)
    finally:
        set_size(768, 384, False)
    if cache:
        os.makedirs(os.path.dirname(p), exist_ok=True)
        np.save(p, img)
    return img


def up(img):
    return img.repeat(W // RW, 0).repeat(W // RW, 1)


# ---- the words ----------------------------------------------------------------------------------------------------

WORDS = {
    # 1: a C library of the time: a prefix, upper-case constants, the arguments a 1997 SDK took
    "1": dict(renderer="Hardware 3D", code="""hw3dInit();
hw3dSelectBoard(0);
hw3dOpenScreen(HW3D_RES_640x480,
               HW3D_REFRESH_60HZ,
               HW3D_COLOR_RGB565,
               HW3D_ORIGIN_UPPER_LEFT, 2, 1);
hw3dTexFilter(HW3D_TMU0,
              HW3D_FILTER_BILINEAR,
              HW3D_FILTER_BILINEAR);
hw3dDither(HW3D_DITHER_4x4);
hw3dFogMode(HW3D_FOG_TABLE);""", brag=["640 x 480", "65,536 COLORS"]),
    # 2: a context-and-enums API, the purple fog's own colour in it (the drive's sky_low, 44 26 62)
    "2": dict(renderer="3D Accelerator", code="""acCreateContext(&ctx, AC_DEVICE_0);
acSetDisplayMode(ctx, 640, 480, 16, 60);
acEnable(ctx, AC_DEPTH_TEST | AC_DITHER);
acTexParameter(ctx, AC_MIN_FILTER, AC_LINEAR);
acTexParameter(ctx, AC_MAG_FILTER, AC_LINEAR);
acFogColor(ctx, 0x2C1A3E);
acFogMode(ctx, AC_FOG_TABLE);
acSwapBuffers(ctx);""", brag=["640 x 480", "16-BIT  BILINEAR", "3D ACCELERATED"]),
    # 3: the game's console, typed: the commands the menu just set (a Quake-era console)
    "3": dict(renderer="Accelerated", code="""] vid_renderer hardware
] vid_mode 640 480 16
] r_texturefilter bilinear
] r_dither 1
] r_fog 1
] r_fogcolor 44 26 62
] r_flares 1
] vid_restart""", brag=["640 x 480", "16-BIT  BILINEAR", "HARDWARE 3D"]),
}


def font(name, size):
    return ImageFont.truetype(SUP + name, size)


PX, PY, PW, PH = 348, 150, 840, 470                        # the panel (glide.py's, 80 wider for the longer values)
ROW_Y = [PY + 130 + 70 * i for i in range(4)]
VAL_X = PX + 430
VAL_W = 350
APPLY = (PX + PW - 230, PY + PH - 80, PX + PW - 40, PY + PH - 30)


def rows(words):
    return [("RENDERER", "Software", WORDS[words]["renderer"]), ("RESOLUTION", "320 x 200", "640 x 480"),
            ("COLOR DEPTH", "8-bit", "16-bit"), ("TEXTURE FILTER", "Nearest", "Bilinear")]


def menu(base, values, hover=None, pressed=False):
    """glide.py's VIDEO OPTIONS, its look kept; the value boxes wider."""
    im = Image.fromarray(base.astype(np.uint8)).convert("RGBA")
    over = Image.new("RGBA", im.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(over)
    d.rectangle([PX, PY, PX + PW, PY + PH], fill=(10, 12, 30, 215), outline=(200, 170, 60, 255), width=4)
    d.text((PX + 40, PY + 30), "VIDEO OPTIONS", font=font("Arial Black.ttf", 52), fill=(255, 205, 60, 255))
    f = font("Arial Bold.ttf", 34)
    for i, (label, a, b) in enumerate(ROWS_NOW):
        y = ROW_Y[i]
        d.text((PX + 50, y), label, font=f, fill=(170, 180, 210, 255))
        box = [VAL_X - 14, y - 6, VAL_X + VAL_W, y + 44]
        d.rectangle(box, fill=(60, 50, 10, 255) if hover == i else (25, 28, 55, 255), outline=(110, 110, 150, 255))
        d.text((VAL_X, y), "< " + values[i] + " >", font=f, fill=(255, 255, 255, 255))
    l, t, r, b = APPLY
    d.rectangle([l, t, r, b], fill=(200, 170, 60, 255) if pressed else (90, 75, 20, 255), outline=(255, 220, 90, 255), width=3)
    d.text((l + 38, t + 6), "APPLY", font=f, fill=(10, 10, 20, 255) if pressed else (255, 230, 120, 255))
    return np.array(Image.alpha_composite(im, over).convert("RGB"))


ROWS_NOW = rows("1")
ARROW = ["X", "XX", "X.X", "X..X", "X...X", "X....X", "X.....X", "X......X", "X.......X", "X........X", "X.....XXXXX",
         "X..X..X", "X.X X..X", "XX  X..X", "X    X..X", "     X..X", "      XX"]


def pointer(img, x, y):
    s = 3
    for j, row in enumerate(ARROW):
        for i, c in enumerate(row):
            if c in "X.":
                img[int(y) + j * s:int(y) + j * s + s, int(x) + i * s:int(x) + i * s + s] = 0 if c == "X" else 255


def code(img, words, level):
    """The calls, white with a drop shadow over the drive (glide.py's), faded by level."""
    if level <= 0:
        return img
    lines = WORDS[words]["code"].split("\n")
    lay = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(lay)
    f = ImageFont.truetype(os.path.join(HERE, "pcfonts/Px437_IBM_VGA_8x16.ttf"), 32)
    y0 = 150 if len(lines) <= 9 else 110
    for i, line in enumerate(lines):
        d.text((120, y0 + 44 * i), line, font=f, fill=255)
    ink = np.array(lay) > 127
    out = img.astype(np.float32).copy()
    sh = np.roll(np.roll(ink, 3, 0), 3, 1)
    out[sh] = out[sh] * (1 - level)
    out[ink] = out[ink] + (255 - out[ink]) * level
    return out


def brag(img, words):
    """glide.py's brag box, bottom left."""
    im = Image.fromarray(np.clip(img, 0, 255).astype(np.uint8))
    d = ImageDraw.Draw(im)
    f = font("Arial Black.ttf", 40)
    lines = WORDS[words]["brag"]
    wmax = max(d.textlength(s, font=f) for s in lines)
    x0, y0 = 60, 520 + 56 * (3 - len(lines))
    d.rectangle([x0 - 20, y0 - 14, x0 + wmax + 20, y0 + 56 * len(lines) + 4], fill=(0, 0, 0))
    for i, s in enumerate(lines):
        d.text((x0, y0 + 56 * i), s, font=f, fill=(255, 255, 255))
    return np.array(im, np.float32)


# ---- output -------------------------------------------------------------------------------------------------------

def writer(out, small):
    size = "1024x512" if small else f"{W}x{H}"
    return subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}",
                             "-r", str(FPS), "-i", "-", "-vf", f"scale={size}:flags=neighbor", "-c:v", "libx264",
                             "-profile:v", "main", "-level", "4.0", "-bf", "0", "-g", "12", "-crf", "18" if small else "16",
                             "-pix_fmt", "yuv420p", "-movflags", "+faststart", out], stdin=subprocess.PIPE)


def beat(put, look, words):
    """glide.py's beat with the new words and the drive at size."""
    global ROWS_NOW
    ROWS_NOW = rows(words)
    for f in range(SOFT):
        put(soft_frame(f).repeat(8, 0).repeat(8, 1))
    # the drive goes on under the menu, dimmed
    shown = [SOFT]
    values = [r[1] for r in ROWS_NOW]
    m = [1300.0, 700.0]

    def glide_to(x, y, k=4):
        x0, y0 = m
        for j in range(1, k + 1):
            u = 1 - (1 - j / k) ** 3
            m[0], m[1] = x0 + (x - x0) * u, y0 + (y - y0) * u
            yield

    def show(hover=None, pressed=False):
        dim = (soft_frame(shown[0]).repeat(8, 0).repeat(8, 1) * 0.55).astype(np.uint8)
        shown[0] += 1
        img = menu(dim, values, hover, pressed)
        pointer(img, *m)
        put(img)
    for _ in range(8):
        show()
    for i in range(4):
        for _ in glide_to(VAL_X + VAL_W - 20, ROW_Y[i] + 18, 3 if i else 5):
            show(hover=i)
        show(hover=i)
        values[i] = ROWS_NOW[i][2]
        show(hover=i)
        for _ in range(4 if i == 0 else 2):
            show(hover=i)
    l, t, r, b = APPLY
    for _ in glide_to((l + r) / 2, (t + b) / 2, 4):
        show()
    show(pressed=True); show(pressed=True); show()
    for _ in range(5):
        put(np.zeros((H, W, 3)))
    half(put, look, words)


BRAG = 82                                                  # the brag from this frame of the accelerated half on


def half(put, look, words):
    """The accelerated half: the calls over the drive 2 s, fading over one; the jump; the brag to the end."""
    for k, f in enumerate(range(HD0, END)):
        img = code(up(hd_frame(f, look)), words, min(1.0, max(0.0, (36 - k) / 12)))
        if k >= BRAG:
            img = brag(img, words)
        put(img)


def main():
    mode = sys.argv[1]
    if mode == "stills":
        global ROWS_NOW
        words, out = sys.argv[2], sys.argv[3]
        ROWS_NOW = rows(words)
        dim = (soft_frame(SOFT + MENU - 1).repeat(8, 0).repeat(8, 1) * 0.55).astype(np.uint8)
        a = menu(dim, [r[2] for r in ROWS_NOW])
        pointer(a, (APPLY[0] + APPLY[2]) / 2, (APPLY[1] + APPLY[3]) / 2)
        b = code(up(hd_frame(HD0 + 6, "A")), words, 1.0)
        c = brag(up(hd_frame(HD0 + 90, "A")), words)
        sheet = np.concatenate([np.concatenate([a, b], 1), np.concatenate([c, np.zeros_like(c)], 1)], 0)
        Image.fromarray(np.clip(sheet, 0, 255).astype(np.uint8)).resize((W, H), Image.BOX).save(out)
        return
    small = sys.argv[-1] == "small"
    if mode == "hd":
        look, out = sys.argv[2], sys.argv[3]
        ff = writer(out, small)
        for f in range(HD0, END):
            ff.stdin.write(up(hd_frame(f, look)).tobytes())
    elif mode == "half":
        look, words, out = sys.argv[2], sys.argv[3], sys.argv[4]
        ff = writer(out, small)
        half(lambda img: ff.stdin.write(np.clip(img, 0, 255).astype(np.uint8).tobytes()), look, words)
    else:
        look, words, out = sys.argv[2], sys.argv[3], sys.argv[4]
        ff = writer(out, small)
        beat(lambda img: ff.stdin.write(np.clip(img, 0, 255).astype(np.uint8).tobytes()), look, words)
    ff.stdin.close()
    ff.wait()


if __name__ == "__main__":
    main()
