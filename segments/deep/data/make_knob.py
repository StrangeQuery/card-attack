#!/usr/bin/env python3
"""The dive's ending at 1536x768 (the volume knob, the design in prototypes/knob.py): its three models as the
mesh kit's cards and the atlas of their surfaces.

    python make_knob.py         writes knob_rock.txt, knob_plate.txt, knob_knob.txt and knob.bin beside it

The models, in the dial's own frame (x across, toward the dial's right as the diver sees it; y up; z away from the
diver along the ground; its middle the plate's top middle), whole units:
- knob_rock.txt: the rock pillar, 30 units to the dive's unit: its craggy side (96 round, 12 rows from the plate's
  foot 25 units down, its radius the prototype's rock_r: lobes, crags and ledges) and its top round the plate (5
  rings out to the side's broken outline);
- knob_plate.txt: the dial plate, 100 units to the dive's unit: its face (a disc in 9 rings, its rim chipped where
  the prototype's is) and its rim's side;
- knob_knob.txt: the knob with its skirt, 100 units to the unit, as it stands at 9 (its pointer 120 degrees round
  from the scale's top): the skirt's side and top, the knob's side and top.
Each corner carries, where the kit keeps a corner's normal, its place in the atlas: across, down, and the page.

The atlas (knob.bin, built data like the dive's other atlases; 11 bytes a texel): for each page its texels, each
the surface there as the prototype shades it: its colour before the dial powers up (S), the same without the
scale's or pointer's ink (C), what the dial's own glow adds at full strength (B: the warm light falling off with the
distance from the dial, times the surface's own colour and crevices), how much ink covers it (32nds) and whose ink
(the numbers 0..10, 11 the knob's pointer marks, 12 none). The light from the water above and the relief (the bumps,
pits, dents, scratches, crust, barnacles and the crevices between them) are worked out here once, onto the
surfaces, as a lightmap is (id Software's Quake, 1996): the light falls from a fixed way on things that don't move.
What changes from frame to frame (where the camera is, the knob's turn, the glow, the ink lighting up, the haze)
is the machines'.

The pages are listed in knob.txt (made by the same run, with the tables).
"""
import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "prototypes"))
import knob as K  # noqa: E402  the design's own surfaces

# the dial's frame (thousandths, as the machines take it): across, up, along
SIDE_T = (992, 0, 128)
FWD_T = (-128, 0, 992)
side = np.array(SIDE_T, float) / 1000; side /= np.linalg.norm(side)
fwd = np.array(FWD_T, float) / 1000; fwd /= np.linalg.norm(fwd)
up = np.array([0.0, 1.0, 0.0])
L = up * 0.85 - fwd * 0.35 - side * 0.4
L /= np.linalg.norm(L)
G0 = 0.95                       # the dial's glow at 9 (the machines' 95 hundredths)
INK = np.array([255, 225, 160])
ANG0 = math.radians(-K.SWEEP / 2 + K.SWEEP * 9 / 10)    # the pointer at 9
PDIR = side * math.sin(ANG0) + fwd * math.cos(ANG0)
ROCK_S, DIAL_S = 30, 100        # units to the dive's unit
SECT = 96
ROCK_ROWS = [-0.45, -0.8, -1.3, -2.0, -3.0, -4.4, -6.2, -8.5, -11.5, -15.0, -20.0, -25.0]
R_IN = 4.0                      # the rock's top from under the plate

FACE = K.dial_face()
RS = FACE["rockside"]

# the pages: name, across, down
PAGES = [("rock_side", 1600, 800), ("rock_top", 1400, 1400), ("plate_face", 1000, 1000), ("plate_rim", 1000, 20),
         ("skirt_side", 500, 12), ("skirt_top", 520, 520), ("knob_side", 1000, 90), ("knob_top", 400, 400)]
PAGE = {n: i for i, (n, _, _) in enumerate(PAGES)}
EXTENT = {"rock_top": K.ROCK_R * 1.6, "plate_face": K.PLATE_R, "skirt_top": 2.45, "knob_top": K.KNOB_R}


def origin(page):
    """A page's origin: its middle where it is more than 1000 texels across or down (a card's places), else 0."""
    _, w, h = PAGES[PAGE[page]]
    return (w // 2 if w > 1000 else 0, h // 2 if h > 1000 else 0)


def to_local(p):
    return np.stack([p @ side, p @ up, p @ fwd], -1)


def to_world(q):
    return q[..., :1] * side + q[..., 1:2] * up + q[..., 2:3] * fwd


def tu_of(th):
    return np.clip(((th / (2 * math.pi) + 0.5) * 1023).astype(int), 0, 1023)


def rock_r(th, y):
    """The prototype's rock radius (its lobes, crags and ledges) at a world angle and height."""
    th = np.arctan2(np.sin(th), np.cos(th))
    tv = np.clip(((-y) * 14).astype(int) % 1024, 0, 1023)
    crag = RS[tv, tu_of(th)] - 0.5
    ledge = 0.35 * np.sign(np.sin(y * 1.1 + 2 * np.sin(th * 2))) * (np.abs(np.sin(y * 1.1 + 2 * np.sin(th * 2))) > 0.7)
    return K.ROCK_R * (1 + 0.12 * np.sin(3 * th + 0.4) + 0.07 * np.sin(7 * th + 0.25 * y)) + 1.6 * crag + ledge - 0.06 * y


def plate_r(th):
    th = np.arctan2(np.sin(th), np.cos(th))
    return K.PLATE_R - 0.25 * np.clip((RS[600, tu_of(th)] - 0.55) * 4, 0, 1)


# --- the prototype's shading, at points of the surfaces ---------------------------------------------------------

def uv(p, scale, size):
    fx = (p[..., 0] * side[0] + p[..., 2] * side[2]) / scale
    fz = (p[..., 0] * fwd[0] + p[..., 2] * fwd[2]) / scale
    return (np.clip(((1 - (fz + 1) / 2) * (size - 1)).astype(int), 0, size - 1), np.clip(((fx + 1) / 2 * (size - 1)).astype(int), 0, size - 1))


def tex(mp, key, iv, iu):
    a = mp[key]
    S = a.shape[0] - 1
    return a[np.clip(iv, 0, S), np.clip(iu, 0, S)]


def fouled(base, mp, iv, iu):
    cr, ru, ve, ba = (tex(mp, k, iv, iu)[..., None] for k in ("crust", "rust", "verd", "barn"))
    c = base * (1 - 0.75 * ru) + np.array([70, 40, 25]) * 0.75 * ru
    c = c * (1 - 0.6 * ve) + np.array([40, 66, 56]) * 0.6 * ve
    c = c * (1 - 0.85 * cr) + np.array([82, 77, 66]) * (0.75 + 0.5 * tex(mp, "h", iv, iu)[..., None]) * 0.85 * cr
    c = c * (1 - 0.9 * ba) + np.array([118, 112, 100]) * 0.9 * ba
    return c


def bumped(mp, iv, iu, k):
    du, dv = tex(mp, "gx", iv, iu)[..., None] * k, tex(mp, "gz", iv, iu)[..., None] * k
    nn = up - du * side - dv * fwd
    return nn / np.linalg.norm(nn, axis=-1, keepdims=True)


def near_glow(p):
    return np.exp(-np.maximum(np.hypot(p[..., 0], p[..., 2]) - 3.0, 0) / 2.2) * np.exp(np.minimum(p[..., 1], 0) / 3.0)


def parts(albedo, nrm, cav, glow_near):
    """The prototype's shade split in two: the water's light (S0) and the dial's glow at strength 1 (B)."""
    diff = np.clip(nrm @ L, 0, 1)[..., None]
    amb = 0.1 + 0.08 * np.clip(nrm @ up, 0, 1)[..., None]
    grey = albedo.mean(-1, keepdims=True)
    dull = grey + 0.45 * (albedo - grey)
    murk = np.array([0.7, 0.9, 1.0])
    keep = (1 - 0.8 * cav[..., None])
    return dull * murk * (amb + 0.42 * diff) * keep, dull * np.array([1.0, 0.8, 0.55]) * glow_near[..., None] ** 2 * 0.5 * keep


def page_grid(name):
    _, w, h = PAGES[PAGE[name]]
    v, u = np.mgrid[0:h, 0:w]
    return u, v, w, h


def planar(name):
    """A flat page's points (local across and along) for its texels."""
    u, v, w, h = page_grid(name)
    e = EXTENT[name]
    return (2 * (u + 0.5) / w - 1) * e, (1 - 2 * (v + 0.5) / h) * e


def surface_out(S0, B, glow_scale=1.0):
    return S0 + G0 * B, B


def bake():
    """Each page's texels: S, C, B (rgb), ink (32nds), whose."""
    pages = []
    # the rock's side: round (world angle) and down (by the rows)
    u, v, w, h = page_grid("rock_side")
    th = (u + 0.5) / w * 2 * math.pi
    rows_v = np.linspace(0, h - 1, len(ROCK_ROWS))
    y = np.interp(v + 0.0, rows_v, ROCK_ROWS)
    R0 = rock_r(th, y)
    e = 0.02
    dRth = (rock_r(th + e, y) - R0) / e
    dRy = (rock_r(th, y + 0.05) - R0) / 0.05
    radial = np.stack([np.cos(th), np.zeros_like(th), np.sin(th)], -1)
    tang = np.stack([-np.sin(th), np.zeros_like(th), np.cos(th)], -1)
    nrm = radial - (dRth / np.maximum(R0, 1))[..., None] * tang - dRy[..., None] * up
    nrm /= np.linalg.norm(nrm, axis=-1, keepdims=True)
    tha = np.arctan2(np.sin(th), np.cos(th))
    cav = np.clip((0.5 - RS[np.clip(((-y) * 14).astype(int) % 1024, 0, 1023), tu_of(tha)]) * 2.5, 0, 1)
    rockc = np.array([72, 62, 52]) * (0.8 + 0.4 * RS[np.clip((y * -41).astype(int) % 1024, 0, 1023), np.clip(((tha / (2 * math.pi) + 0.5) * 1023 * 3).astype(int) % 1024, 0, 1023)])[..., None]
    p = np.stack([R0 * np.cos(th), y, R0 * np.sin(th)], -1)
    S0, B = parts(rockc, nrm, cav, near_glow(p))
    pages.append((S0 + G0 * B, S0 + G0 * B, B, np.zeros(th.shape), np.full(th.shape, 12)))
    # the rock's top round the plate
    lx, lz = planar("rock_top")
    p = to_world(np.stack([lx, np.full(lx.shape, -K.PLATE_T), lz], -1))
    rr = np.hypot(p[..., 0], p[..., 2])
    iv, iu = uv(p, K.ROCK_R * 1.6, 1024)
    mp = FACE["rock"]
    weed = (FACE["weed"][iv, iu] * (rr > K.PLATE_R + 0.2))[..., None]
    alb = fouled(np.array([78, 66, 54]), mp, iv, iu)
    alb = alb * (1 - weed) + np.array([30, 52, 24]) * weed
    contact = 1 - 0.7 * np.exp(-np.maximum(rr - K.PLATE_R, 0) / 0.5)[..., None]
    S0, B = parts(alb, bumped(mp, iv, iu, 1.6), tex(mp, "cav", iv, iu), near_glow(p))
    S0, B = S0 * contact, B * contact
    pages.append((S0 + G0 * B, S0 + G0 * B, B, np.zeros(rr.shape), np.full(rr.shape, 12)))
    # the plate's face, its scale
    lx, lz = planar("plate_face")
    p = to_world(np.stack([lx, np.zeros(lx.shape), lz], -1))
    iv, iu = uv(p, K.PLATE_R, 1024)
    mp = FACE["plate"]
    numb = FACE["num"][iv, iu]
    ink = FACE["ink"][iv, iu]
    S0, B = parts(fouled(np.array([46, 47, 52]), mp, iv, iu), bumped(mp, iv, iu, 1.3), tex(mp, "cav", iv, iu), near_glow(p))
    c = S0 + G0 * B
    g = np.clip(ink, 0, 1.6)[..., None]
    stain = (0.55 + 0.45 * tex(mp, "rust", iv, iu))[..., None]
    paint = np.array([150, 140, 115]) * (1 - stain * 0.4) * 0.5
    fade = 0.35
    S = c * (1 - np.clip(g, 0, 1) * 0.7) + (paint * (1 - fade) + INK * G0 * 0.8 * fade) * g * (0.45 + 0.55 * fade)
    pages.append((S, c, B, ink, np.where((numb >= 0) & (ink > 0), numb, 12)))
    # the plate's rim
    u, v, w, h = page_grid("plate_rim")
    th = (u + 0.5) / w * 2 * math.pi
    y = -(v + 0.5) / h * K.PLATE_T
    R = plate_r(th)
    p = np.stack([R * np.cos(th), y, R * np.sin(th)], -1)
    iv, iu = uv(p * 0.98, K.PLATE_R, 1024)
    nrm = np.stack([np.cos(th), np.zeros_like(th), np.sin(th)], -1)
    S0, B = parts(fouled(np.array([66, 66, 72]), mp, iv, iu), nrm, tex(mp, "cav", iv, iu), near_glow(p))
    pages.append((S0 + G0 * B, S0 + G0 * B, B, np.zeros(th.shape), np.full(th.shape, 12)))
    # the knob, as it stands at 9 (its maps the prototype's, laid on in the world as it stands then)
    SK, SKH = 2.45, 0.14
    mp = FACE["skirt"]
    u, v, w, h = page_grid("skirt_side")
    phi = (u + 0.5) / w * 2 * math.pi
    y = SKH * (1 - (v + 0.5) / h)
    p = to_world(np.stack([SK * np.sin(phi), y, SK * np.cos(phi)], -1))
    nrm = np.stack([p[..., 0], np.zeros_like(y), p[..., 2]], -1) / SK
    iv, iu = uv(p, SK, 512)
    S0, B = parts(fouled(np.array([60, 60, 64]), mp, iv, iu), nrm, tex(mp, "cav", iv, iu), near_glow(p))
    pages.append((S0 + G0 * B, S0 + G0 * B, B, np.zeros(y.shape), np.full(y.shape, 12)))
    lx, lz = planar("skirt_top")
    p = to_world(np.stack([lx, np.full(lx.shape, SKH), lz], -1))
    al = p[..., 0] * PDIR[0] + p[..., 2] * PDIR[2]
    ac = np.abs(p[..., 0] * PDIR[2] - p[..., 2] * PDIR[0])
    iv, iu = uv(p, SK, 512)
    sline = ((al > K.KNOB_R * 0.8) & (ac < 0.11)).astype(float) * (1 - tex(mp, "crust", iv, iu) * 0.7)
    S0, B = parts(fouled(np.array([44, 44, 48]), mp, iv, iu), bumped(mp, iv, iu, 1.2), tex(mp, "cav", iv, iu), near_glow(p))
    c = S0 + G0 * B
    S = c * (1 - sline[..., None]) + INK * G0 * 0.4 * sline[..., None]
    pages.append((S, c, B, sline, np.where(sline > 0, 11, 12)))
    mp = FACE["side"]
    u, v, w, h = page_grid("knob_side")
    phi = (u + 0.5) / w * 2 * math.pi
    y = SKH + (K.KNOB_H - SKH) * (1 - (v + 0.5) / h)
    p = to_world(np.stack([K.KNOB_R * np.sin(phi), y, K.KNOB_R * np.cos(phi)], -1))
    ph = np.arctan2(p[..., 2], p[..., 0])
    sv = np.clip(((ph / (2 * math.pi) + 0.5) * 511).astype(int), 0, 511)
    sh = np.clip(((y / K.KNOB_H) * 511).astype(int), 0, 511)
    worn = np.clip((tex(mp, "h", sh // 4 + 100, sv) + 0.3) * 1.5, 0, 1)
    ridge = np.sin(ph * 36)
    radial = np.stack([np.cos(ph), np.zeros_like(ph), np.sin(ph)], -1)
    tang = np.stack([-np.sin(ph), np.zeros_like(ph), np.cos(ph)], -1)
    nrm = radial + (0.8 * np.cos(ph * 36) * (1 - worn))[..., None] * tang
    nrm /= np.linalg.norm(nrm, axis=-1, keepdims=True)
    notch = ((radial @ PDIR) > 0.985).astype(float)
    alb = fouled(np.array([48, 48, 52]) * (0.85 + 0.15 * ridge[..., None]), mp, sh, sv)
    S0, B = parts(alb, nrm, tex(mp, "cav", sh, sv) + 0.3 * (1 - worn) * (ridge < -0.6), near_glow(p) * 0.6)
    c = S0 + G0 * B
    S = np.where(notch[..., None] > 0, INK * G0 * 0.4, c)
    pages.append((S, c, B, notch, np.where(notch > 0, 11, 12)))
    lx, lz = planar("knob_top")
    p = to_world(np.stack([lx, np.full(lx.shape, K.KNOB_H), lz], -1))
    along = p[..., 0] * PDIR[0] + p[..., 2] * PDIR[2]
    across = np.abs(p[..., 0] * PDIR[2] - p[..., 2] * PDIR[0])
    line = ((along > 0.1) & (across < 0.13)).astype(float)
    mp = FACE["top"]
    iv, iu = uv(p, K.KNOB_R, 512)
    S0, B = parts(fouled(np.array([50, 50, 54]), mp, iv, iu), bumped(mp, iv, iu, 1.4), tex(mp, "cav", iv, iu), near_glow(p) * 0.5)
    c = S0 + G0 * B
    lw = line * (1 - tex(mp, "crust", iv, iu) * 0.6)
    S = c * (1 - lw[..., None]) + INK * G0 * 0.4 * lw[..., None]
    pages.append((S, c, B, lw, np.where(lw > 0, 11, 12)))
    return pages


# --- the models ---------------------------------------------------------------------------------------------------

def unit99(n):
    n = n / np.maximum(np.linalg.norm(n, axis=-1, keepdims=True), 1e-12)
    return np.rint(99 * n).astype(int)


class Mesh:
    def __init__(self, scale):
        self.scale, self.v, self.c, self.t, self.out = scale, [], [], [], []

    def corner(self, q, page, a, b):
        """A corner at a local place (the dive's units), with its place in the atlas (from the page's origin, so
        every number is a card)."""
        self.v.append(np.rint(np.asarray(q) * self.scale).astype(int))
        ou, ov = origin(page)
        self.c.append((a - ou, b - ov, PAGE[page]))
        return len(self.v) - 1

    def tri(self, i, j, k, outward):
        self.t.append((i, j, k))
        self.out.append(np.asarray(outward, float))

    def quad(self, a, b, c, d, outward):
        self.tri(a, b, c, outward)
        self.tri(a, c, d, outward)

    def write(self, path, what):
        V = np.array(self.v)
        lines = [f"# {what}. Made by make_knob.py.", f"faces {len(self.t)}", f"vertices {len(V)}"]
        for p, c in zip(V, self.c):
            assert np.all(np.abs(p) <= 999)
            lines.append(f"{p[0]} {p[1]} {p[2]} {c[0]} {c[1]} {c[2]}")
        lines.append(f"triangles {len(self.t)} nouv")
        for (i, j, k), o in zip(self.t, self.out):
            n = np.cross(V[j] - V[i], V[k] - V[i]).astype(float)
            if np.linalg.norm(n) < 1e-9:
                n = o
            if n @ o < 0:
                n = -n
            n = unit99(n)
            d = int(np.rint((n * V[i]).sum() / 1000))
            lines.append(f"{i} {j} {k} {n[0]} {n[1]} {n[2]} {d}")
        open(path, "w").write("\n".join(lines) + "\n")
        print(f"{path}: {len(V)} corners, {len(self.t)} triangles")


def texel_planar(name, q):
    """A local place's texel on a flat page (across, down)."""
    _, w, h = PAGES[PAGE[name]]
    e = EXTENT[name]
    a = int(np.clip(np.floor((q[0] / e + 1) / 2 * w), 0, w - 1))
    b = int(np.clip(np.floor((1 - q[2] / e) / 2 * h), 0, h - 1))
    return a, b


def models():
    angles = [2 * math.pi * k / SECT for k in range(SECT + 1)]
    # the rock
    m = Mesh(ROCK_S)
    _, w, h = PAGES[PAGE["rock_side"]]
    grid = []
    for r, y in enumerate(ROCK_ROWS):
        row = []
        for k, th in enumerate(angles):
            R = float(rock_r(np.array(th), np.array(y)))
            q = to_local(np.array([R * math.cos(th), y, R * math.sin(th)]))
            row.append(m.corner(q, "rock_side", round(k / SECT * (w - 1)), round(r / (len(ROCK_ROWS) - 1) * (h - 1))))
        grid.append(row)
    for r in range(len(ROCK_ROWS) - 1):
        for k in range(SECT):
            th = (angles[k] + angles[k + 1]) / 2
            m.quad(grid[r][k], grid[r + 1][k], grid[r + 1][k + 1], grid[r][k + 1], to_local(np.array([math.cos(th), 0, math.sin(th)])))
    rings = []
    for j, fr in enumerate([0, 0.25, 0.5, 0.75, 1.0]):
        ring = []
        for k, th in enumerate(angles[:SECT]):
            Ro = float(rock_r(np.array(th), np.array(-K.PLATE_T)))
            r = R_IN + (Ro - R_IN) * fr
            q = to_local(np.array([r * math.cos(th), -K.PLATE_T, r * math.sin(th)]))
            ring.append(m.corner(q, "rock_top", *texel_planar("rock_top", q)))
        rings.append(ring)
    for j in range(4):
        for k in range(SECT):
            k1 = (k + 1) % SECT
            m.quad(rings[j][k], rings[j][k1], rings[j + 1][k1], rings[j + 1][k], (0, 1, 0))
    m.write(os.path.join(HERE, "knob_rock.txt"), "the knob's rock pillar, 30 units to the dive's unit, in the dial's frame")
    # the plate
    m = Mesh(DIAL_S)
    centre = m.corner(np.zeros(3), "plate_face", *texel_planar("plate_face", np.zeros(3)))
    rings = []
    for fr in [0.6, 1.2, 1.8, 2.45, 3.0, 3.5, 4.0, 4.3, None]:
        ring = []
        for th in angles[:SECT]:
            r = float(plate_r(np.array(th))) if fr is None else fr
            q = to_local(np.array([r * math.cos(th), 0.0, r * math.sin(th)]))
            ring.append(m.corner(q, "plate_face", *texel_planar("plate_face", q)))
        rings.append(ring)
    for k in range(SECT):
        m.tri(centre, rings[0][k], rings[0][(k + 1) % SECT], (0, 1, 0))
    for j in range(len(rings) - 1):
        for k in range(SECT):
            k1 = (k + 1) % SECT
            m.quad(rings[j][k], rings[j][k1], rings[j + 1][k1], rings[j + 1][k], (0, 1, 0))
    _, w, h = PAGES[PAGE["plate_rim"]]
    top, bot = [], []
    for k, th in enumerate(angles):
        r = float(plate_r(np.array(th)))
        for y, lst, b in ((0.0, top, 0), (-K.PLATE_T, bot, h - 1)):
            lst.append(m.corner(to_local(np.array([r * math.cos(th), y, r * math.sin(th)])), "plate_rim", round(k / SECT * (w - 1)), b))
    for k in range(SECT):
        th = (angles[k] + angles[k + 1]) / 2
        m.quad(top[k], bot[k], bot[k + 1], top[k + 1], to_local(np.array([math.cos(th), 0, math.sin(th)])))
    m.write(os.path.join(HERE, "knob_plate.txt"), "the dial's plate, 100 units to the dive's unit, in the dial's frame")
    # the knob and its skirt, at 9: places round it by the angle phi from the dial's along toward its across
    m = Mesh(DIAL_S)
    SK, SKH = 2.45, 0.14
    loc = lambda r, y, phi: np.array([r * math.sin(phi), y, r * math.cos(phi)])
    def band(page, r, ys, vs, outward):
        _, w, h = PAGES[PAGE[page]]
        rows = [[m.corner(loc(r, y, phi), page, round(k / SECT * (w - 1)), vv) for k, phi in enumerate(angles)] for y, vv in zip(ys, vs)]
        for a in range(len(rows) - 1):
            for k in range(SECT):
                phi = (angles[k] + angles[k + 1]) / 2
                m.quad(rows[a][k], rows[a + 1][k], rows[a + 1][k + 1], rows[a][k + 1], outward(phi))
    band("skirt_side", SK, [SKH, 0.0], [0, 11], lambda phi: (math.sin(phi), 0, math.cos(phi)))
    band("knob_side", K.KNOB_R, [K.KNOB_H, (K.KNOB_H + SKH) / 2, SKH], [0, 45, 89], lambda phi: (math.sin(phi), 0, math.cos(phi)))
    def disc(page, rs, y):
        rings = []
        for r in rs:
            if r == 0:
                q = np.array([0.0, y, 0.0])
                rings.append([m.corner(q, page, *texel_planar(page, q))] * SECT)
                continue
            rings.append([m.corner(q, page, *texel_planar(page, q)) for q in (loc(r, y, phi) for phi in angles[:SECT])])
        for j in range(len(rings) - 1):
            for k in range(SECT):
                k1 = (k + 1) % SECT
                if rs[j] == 0:
                    m.tri(rings[0][0], rings[1][k], rings[1][k1], (0, 1, 0))
                else:
                    m.quad(rings[j][k], rings[j][k1], rings[j + 1][k1], rings[j + 1][k], (0, 1, 0))
    disc("skirt_top", [K.KNOB_R, 2.1, SK], SKH)
    disc("knob_top", [0, 0.6, 1.2, K.KNOB_R], K.KNOB_H)
    m.write(os.path.join(HERE, "knob_knob.txt"), "the knob and its skirt at 9, 100 units to the dive's unit, in the dial's frame (along: the pointer at 0)")


# the sticky turn (prototypes/knob.py): frame, the dial's value (hundredths) and the knob's shudder (hundredths of a
# unit across)
TURN = {308: (900, 0), 309: (900, 0), 310: (900, 0), 311: (900, 0), 312: (900, 0), 313: (907, 5), 314: (911, -4),
        315: (904, 3), 316: (906, -3), 317: (904, 2), 318: (905, -1), 319: (904, 0), 320: (904, 0), 321: (904, 0),
        322: (914, 6), 323: (918, -5), 324: (910, 4), 325: (912, -3), 326: (910, 2), 327: (911, -2), 328: (910, 1),
        329: (910, 0), 330: (962, 0)}


def tables(at):
    """knob.txt: the pages, the turn, the printed tables and the random cards."""
    L = ["# The dive's ending at 1536x768: the knob's atlas pages and tables. Made by make_knob.py."]
    L.append("pages " + " ".join(f"{w} {h} {a} {origin(n)[0]} {origin(n)[1]}" for (n, w, h), a in zip(PAGES, at)))
    L.append("turn " + " ".join(f"{n} {v} {w}" for n, (v, w) in sorted(TURN.items())))
    # the orca's haze against the lit water (the prototype's reach 120): 32 min(1, 0.85 (a / 120)^0.8), a 0..199
    L.append("haze120 " + " ".join(str(round(32 * min(1.0, 0.85 * (a / 120) ** 0.8))) for a in range(200)))
    # the water's light beyond the dial, by 4 rows of the screen (0..191) and channel: exp(-y / 0.35 768) (12, 22, 26)
    for ch, top in enumerate((12, 22, 26)):
        L.append(f"lift{ch} " + " ".join(str(round(top * math.exp(-(4 * t + 2) / (0.35 * 768)))) for t in range(192)))
    # the dial's glow on the water and the silt's puffs, by tenths of e (256ths): 25.6 exp(-e), 256 exp(-e)
    L.append("halo " + " ".join(str(round(25.6 * math.exp(-e / 10))) for e in range(100)))
    L.append("puff " + " ".join(str(round(256 * math.exp(-e / 10))) for e in range(100)))
    # the lightning's glow (32nds) by whole pixels along past a line's end (0..63) and across it (0..63), for its
    # widths 2, 4 and 5: the prototype's line blurred (sigma 4 + 2 w) three times over,
    # min(1, 3 w / (sigma sqrt(2 pi)) exp(-(s^2 + t^2) / 2 sigma^2)); s 64 + t
    for w in (2, 4, 5):
        sg = 4 + 2 * w
        L.append(f"glow{w} " + " ".join(str(round(32 * min(1.0, 3 * w / (sg * math.sqrt(2 * math.pi)) * math.exp(-(a * a + b * b) / (2 * sg * sg))))) for a in range(64) for b in range(64)))
    rng = np.random.default_rng(2026_10_26)
    card = lambda n: " ".join(str(int(v)) for v in rng.integers(0, 1000, n))
    # the silt's blobs, 3 numbers each: the angle round the plate (millionths of a turn, 1000 c), how far out (120 c)
    # and the rise (c), from a card c each
    silt = rng.integers(0, 1000, (2 * 18, 3)).astype(np.int64) * [1000, 120, 1]
    L.append("silt " + " ".join(str(int(v)) for v in silt.ravel()))
    # the flakes, 7 numbers each from a card c each: when (thousandths of a frame, 18 c), the angle (millionths of a
    # turn, 1000 c), how far from the middle (300 c), the speed (17 c), the rise (10 c), pale (1 for 45 in 100, a
    # flag) and the size (45 c)
    deb = rng.integers(0, 1000, (140, 7)).astype(np.int64)
    deb[:, 5] = rng.random(140) < 0.45
    deb *= [18, 1000, 300, 17, 10, 1, 45]
    L.append("debris " + " ".join(str(int(v)) for v in deb.ravel()))
    # the knob's shudder, per frame from a card c each: across 2 c - 999 and along 6 (2 c - 999)
    shake = (2 * rng.integers(0, 1000, (21, 2)).astype(np.int64) - 999) * [1, 6]
    L.append("shake " + " ".join(str(int(v)) for v in shake.ravel()))
    # the arcs, 80 numbers each from a card c each: the angle (millionths of a turn, 1000 c), how far out (400 c), the
    # turn (1000ths of nt, (c - 500) 159,155), the length (c), then per step its turn ((c - 500) 222,817), whether it
    # forks (1 for 3 in 10, a flag), the fork's turn ((c - 500) 413,803) and its four steps' turns ((c - 500) 190,986)
    arc = rng.integers(0, 1000, (22 * 20, 80)).astype(np.int64)
    for j in range(10):
        arc[:, 5 + 7 * j] = rng.random(22 * 20) < 0.3
    arc[:, 0] *= 1000
    arc[:, 1] *= 400
    arc[:, 2] = (arc[:, 2] - 500) * 159_155
    for j in range(10):
        b = 4 + 7 * j
        arc[:, b] = (arc[:, b] - 500) * 222_817
        arc[:, b + 2] = (arc[:, b + 2] - 500) * 413_803
        for m in range(4):
            arc[:, b + 3 + m] = (arc[:, b + 3 + m] - 500) * 190_986
    L.append("crackle " + " ".join(str(int(v)) for v in arc.ravel()))
    open(os.path.join(HERE, "knob.txt"), "w").write("\n".join(L) + "\n")


def main():
    models()
    pages = bake()
    with open(os.path.join(HERE, "knob.bin"), "wb") as f:
        at = []
        for (name, w, h), (S, C, B, ink, who) in zip(PAGES, pages):
            at.append(f.tell())
            rec = np.zeros((h, w, 11), np.uint8)
            rec[..., 0:3] = np.clip(np.rint(S), 0, 255)
            rec[..., 3:6] = np.clip(np.rint(C), 0, 255)
            rec[..., 6:9] = np.clip(np.rint(B), 0, 255)
            rec[..., 9] = np.where(who < 12, np.clip(np.rint(np.clip(ink, 0, 1) * 32), 0, 32), 0)
            rec[..., 10] = np.where(rec[..., 9] > 0, who, 12)
            f.write(rec.tobytes())
    tables(at)
    print("knob.bin:", os.path.getsize(os.path.join(HERE, "knob.bin")), "bytes;", " ".join(f"{n} {w} {h} {a}" for (n, w, h), a in zip(PAGES, at)))


if __name__ == "__main__":
    main()
