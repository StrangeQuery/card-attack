"""Prototype: the 90s' network cable. An RJ-45 plug on its blue cable lines up with a phone wall plate's RJ-11
jack and goes for it, about once a second, over and over: it is too wide (8 contacts to the jack's 6) and bounces
off every time, with the same doomed optimism. Isometric, outlined, on black, like the USB stick and the mixers.
Sizes in millimetres, true to the parts: the plug 11.7 wide, the jack's mouth 9.6.

    python rj45.py OUT.rgb [WxH]          about 4.5 s at 12 fps, raw RGB
"""
import math
import sys

import numpy as np
from PIL import Image, ImageDraw

import mtv_aha as M
from mtv import trans, rot_x, rot_y
from mixers import ISO_DIR, prism, circle

FPS = 12
PLATE, HOLE, GOLD = (300, 292, 262), (22, 22, 26), (226, 184, 64)
CLEAR, CABLE, BOOT = (196, 214, 226), (46, 116, 226), (36, 96, 200)
TRIES = 4
CYCLE = 12                                              # frames a try takes: about a second
HOLD = 8                                                # it hangs there after the last bounce, still hopeful
FRAMES = TRIES * CYCLE + HOLD
BACK = 16.0                                             # how far out it waits, nose to the plate

def smoother(x):
    x = min(1.0, max(0.0, x))
    return x * x * x * (x * (6 * x - 15) + 10)

def ease_in(x):
    x = min(1.0, max(0.0, x))
    return x * x

def ease_out(x):
    x = min(1.0, max(0.0, x))
    return 1 - (1 - x) * (1 - x)

# ---- the parts: the wall plate faces +z from z = 0; the plug points its nose along -z --------------------------------

def plate(s):
    s.box(trans(0, 8, -3), (70, 115, 6), PLATE)                                     # the plate, the jack low on it
    s.box(trans(0, 0, 0.05), (9.6, 6.6, 0.1), HOLE)                                  # the jack's mouth
    s.box(trans(0, -3.7, 0.05), (3.2, 1.0, 0.1), HOLE)                               # its keyway, below
    for k in range(4):                                                              # four of its six places wired
        s.box(trans((k - 1.5) * 1.0, 2.4, 0.12), (0.4, 1.2, 0.05), GOLD)
    for x in (-25, 25):                                                             # the screws
        s.box(trans(x, 8, 0.2), (4.0, 4.0, 0.4), (200, 200, 196))
        s.box(trans(x, 8, 0.45), (3.0, 0.5, 0.1), (120, 120, 116))

def plug(s, m):
    """The RJ-45 at m, its nose at m's origin, pointing -z: the clear body, eight gold contacts on top at the front,
    the latch underneath, the boot and the start of the cable behind."""
    s.box(m @ trans(0, 0, 10.5), (11.7, 8.0, 21), CLEAR)
    for k in range(8):
        x = (k - 3.5) * 1.02
        s.box(m @ trans(x, 4.02, 2.2), (0.55, 0.06, 4.0), GOLD)                       # the contacts, on top
        s.box(m @ trans(x, 2.6, -0.03), (0.55, 2.4, 0.06), GOLD)                      # and showing at the nose
    s.box(m @ trans(0, -4.6, 6.0) @ rot_x(math.radians(-14)), (6.0, 0.9, 12), CLEAR)  # the latch
    prism(s, m @ trans(0, 0, 27), circle(4.6, 12), 12, BOOT)                          # the boot
    prism(s, m @ trans(0, 0, 33.5), circle(3.0, 10), 2, CABLE)                        # where the cable leaves it

def tube(s, pts, r, colour):
    """A cable through pts, a prism per segment."""
    for a, b in zip(pts, pts[1:]):
        d = b - a
        L = np.linalg.norm(d)
        z = d / L
        x = np.cross([0, 1.0, 0], z); x /= np.linalg.norm(x)
        y = np.cross(z, x)
        m = np.eye(4); m[:3, :3] = np.stack([x, y, z], 1); m[:3, 3] = (a + b) / 2
        prism(s, m, circle(r, 10), L + 0.6, colour)

# ---- the motion ---------------------------------------------------------------------------------------------------

def try_shape(k):
    """How try k differs: a little off line each time, and the last one with a real wind-up."""
    rng = np.random.default_rng(45 + k)
    last = k == TRIES - 1
    return {"yaw": rng.uniform(-5, 5), "pitch": rng.uniform(-4, 4), "dx": rng.uniform(-0.8, 0.8),
            "dy": rng.uniform(-0.6, 0.6), "wind": 8.0 if last else 0.0, "kick": 1.6 if last else 1.0}

def nose(i):
    """The plug's nose: distance out from the plate, sideways offsets, and its yaw and pitch in degrees; plus how
    long ago it hit (frames), or None."""
    k = min(int(i // CYCLE), TRIES - 1)
    u = (i - k * CYCLE) / CYCLE
    sh = try_shape(k)
    hit_at = 0.42
    if u < hit_at:                                       # wind up, then go for it, faster and faster
        w = sh["wind"] * math.sin(math.pi * min(1, u / 0.2)) if u < 0.2 else 0.0
        z = BACK + w - (BACK) * ease_in(max(0.0, u - (0.2 if sh["wind"] else 0.0)) / (hit_at - (0.2 if sh["wind"] else 0.0)))
        since = None
        wob = 0.0
    else:                                                # bounce off, out past where it waits, and settle back to wait
        v = (u - hit_at) / (1 - hit_at)
        out = BACK * (1 + 0.35 * sh["kick"])
        z = out * ease_out(v / 0.45) if v < 0.45 else lerp(out, BACK, smoother((v - 0.45) / 0.55))
        since = (u - hit_at) * CYCLE
        wob = math.exp(-v * 4) * math.sin(v * 14)
    if i >= TRIES * CYCLE:                               # after the last: just hanging there, waiting
        z, wob, since = BACK, 0.0, None
    a = smoother(u / 0.35) if i < TRIES * CYCLE else 1.0  # each try's aim settles in as it goes
    yaw = sh["yaw"] * (1 - a) + 9 * wob * sh["kick"]
    pitch = sh["pitch"] * (1 - a) - 6 * wob * sh["kick"]
    return z, sh["dx"] * (1 - a), sh["dy"] * (1 - a), yaw, pitch, since

def lerp(a, b, t):
    return a + (b - a) * t

WALL = rot_y(math.radians(-12))                         # the plate's face a little more to the camera than 45 degrees

CABLE_REST = [np.array([0, -4.0 * (k / 6) ** 2 * 6, 34.5 + 7.0 * k]) for k in range(7)]   # sagging off toward the camera

def plug_matrix(i):
    z, dx, dy, yaw, pitch, _ = nose(i)
    return trans(dx, dy, z) @ rot_y(math.radians(yaw)) @ rot_x(math.radians(pitch))

def cable(i):
    """The cable behind the boot: it leaves the boot with the plug and eases toward its far end, which stays put
    off-screen, so it gives and flexes as the plug goes in and bounces."""
    m, still = plug_matrix(i), trans(0, 0, BACK)
    out = []
    for k, rest in enumerate(CABLE_REST):
        w = 0.75 * (k / (len(CABLE_REST) - 1)) ** 1.5
        p = np.array([*rest, 1.0])
        out.append(((1 - w) * (m @ p) + w * (still @ p))[:3])
    return out

# ---- drawing ------------------------------------------------------------------------------------------------------

def outline(img, oid, nrm, izb):
    edge = np.zeros(oid.shape, bool)
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
        o2 = np.roll(oid, (dy, dx), (0, 1)); n2 = np.roll(nrm, (dy, dx), (0, 1))
        edge |= ((o2 != oid) & ((oid == 0) != (o2 == 0))) | ((oid > 0) & (np.abs(n2 - nrm).sum(2) > 0.7))
    out = img.copy()
    out[edge & (izb > 0)] = 24
    sil = izb > 0
    rim = np.zeros(sil.shape, bool)
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
        rim |= np.roll(sil, (dy, dx), (0, 1)) & ~sil
    out[rim] = (24, 24, 28)
    return out

def bonk(im, at, since, H):
    """Impact ticks round where the nose hit, for the two frames after it."""
    if since is None or since > 2.5:
        return
    d = ImageDraw.Draw(im)
    x, y = at
    r0, r1 = H * 0.10, H * (0.17 + 0.03 * since)
    w = max(1, H // 64)
    for a in (-150, -110, -70, -30, 30, 70):
        c, s = math.cos(math.radians(a)), math.sin(math.radians(a))
        d.line([(x + c * r0, y + s * r0), (x + c * r1, y + s * r1)], fill=(255, 255, 255), width=w)

def frame(i, W, H, basis, eye, hfov):
    s = M.Scene()
    plate(s)
    since = nose(i)[5]
    plug(s, plug_matrix(i))
    tube(s, cable(i), 3.0, CABLE)
    R = WALL[:3, :3]                                     # the wall turned away, so the plug comes at it from the side
    s.tris = [(p @ R.T, c, None if n is None else R @ n, uv, ex) for p, c, n, uv, ex in s.tris]
    img, tone, oid, nrm, izb, tag = M.raster(s.tris, eye, basis, hfov, W, H, {})
    img = outline(img, oid, nrm, izb)
    im = Image.fromarray(np.clip(img, 0, 255).astype(np.uint8))
    fpx = (W / 2) / math.tan(hfov / 2)
    c = basis @ (np.array([0.0, 0.0, 0.0]) - eye)
    bonk(im, (W / 2 + fpx * c[0] / c[2], H / 2 - fpx * c[1] / c[2]), since, H)
    return np.asarray(im)

def main():
    out = sys.argv[1]
    W, H = map(int, (sys.argv[2] if len(sys.argv) > 2 else "192x96").split("x"))
    look = ISO_DIR / np.linalg.norm(ISO_DIR)
    centre = WALL[:3, :3] @ np.array([0.0, 7.0, 12.0])
    eye = centre - look * 4000
    basis = M.look(eye, centre)
    hfov = 2 * math.atan(46 / 4000)
    with open(out, "wb") as fh:
        for i in range(FRAMES):
            fh.write(frame(i, W, H, basis, eye, hfov).tobytes())

if __name__ == "__main__":
    main()
