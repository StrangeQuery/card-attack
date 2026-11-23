"""Prototype: the 90s bumper, take 2 (not toasters). Flying stand mixers with feathered wings, an ordinary
unbranded tilt-head mixer modelled in 3-D and rendered isometrically (orthographic, the classic 2:1 pixel-art angle,
crisp dark outlines), flapping as they glide down from the top right across black. No caption: deadpan.

    python mixers.py OUT.rgb [WxH]          120 frames (10 s: 1 s of black, then 9 s of flight) at 12 fps, raw RGB
"""
import math
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

import mtv
import mtv_aha as M
from mtv import trans, rot_x, rot_y, rot_z

FPS, PHASES = 12, 6
PAUSE = 12                                              # a second of black before they come in
IMPACT = "/System/Library/Fonts/Supplemental/Impact.ttf"
RED, STEEL, CHROME, DARK, WHITE = (200, 32, 44), (196, 202, 210), (236, 238, 242), (46, 46, 52), (250, 250, 250)

def prism(s, m, ring, length, colour, cap=None):
    """A prism along z through m: the polygon `ring` (x, y points) swept from z = -length/2 to +length/2, both ends
    closed. cap scales the front ring (rounded-looking noses)."""
    n = len(ring)
    back = [m[:3, :3] @ np.array([x, y, length / 2]) + m[:3, 3] for x, y in ring]
    c = 1.0 if cap is None else cap
    front = [m[:3, :3] @ np.array([x * c, y * c, -length / 2]) + m[:3, 3] for x, y in ring]
    centre_b, centre_f = np.mean(back, 0), np.mean(front, 0)
    for k in range(n):
        j = (k + 1) % n
        q = np.array([back[k], back[j], front[j], front[k]])
        nrm = np.cross(q[1] - q[0], q[3] - q[0]); nrm /= np.linalg.norm(nrm) + 1e-9
        if nrm @ (q.mean(0) - (centre_b + centre_f) / 2) < 0:
            nrm = -nrm
        ex = {"tag": 1}
        s.tris.append((q[[0, 1, 2]], colour, nrm, None, ex)); s.tris.append((q[[0, 2, 3]], colour, nrm, None, ex))
    for pts, centre in ((back, centre_b), (front, centre_f)):
        nrm = np.cross(pts[1] - pts[0], pts[2] - pts[0]); nrm /= np.linalg.norm(nrm) + 1e-9
        if nrm @ (centre - (centre_b + centre_f) / 2) < 0:
            nrm = -nrm
        for k in range(1, n - 1):
            s.tris.append((np.array([pts[0], pts[k], pts[k + 1]]), colour, nrm, None, {"tag": 1}))

def circle(r, n=12, squash=1.0):
    return [(r * math.cos(2 * math.pi * k / n), r * math.sin(2 * math.pi * k / n) * squash) for k in range(n)]

def lathe(s, m, profile, colour, n=18, tag=1):
    """A surface of revolution about m's y axis through (radius, height) profile points."""
    rings = [[m[:3, :3] @ np.array([r * math.cos(2 * math.pi * k / n), y, r * math.sin(2 * math.pi * k / n)]) + m[:3, 3]
              for k in range(n)] for r, y in profile]
    axis = m[:3, 1]
    for a, b in zip(rings, rings[1:]):
        for k in range(n):
            j = (k + 1) % n
            q = np.array([a[k], a[j], b[j], b[k]])
            nrm = np.cross(q[1] - q[0], q[3] - q[0]); nrm /= np.linalg.norm(nrm) + 1e-9
            mid = q.mean(0) - m[:3, 3]
            if nrm @ (mid - axis * (mid @ axis)) < 0:
                nrm = -nrm
            s.tris.append((q[[0, 1, 2]], colour, nrm, None, {"tag": tag}))
            s.tris.append((q[[0, 2, 3]], colour, nrm, None, {"tag": tag}))

FEATHER = (400, 400, 410)                                 # brighter than white, so lighting can't grey it
FEATHER_SHADE = (330, 330, 345)

_FEATHER_ID = [0]

def flat(s, m, pts, colour, tag=None):
    """A flat polygon (u, v points in m's xz plane, a fan from the first point), seen from either side."""
    if tag is None:
        _FEATHER_ID[0] = (_FEATHER_ID[0] + 1) % 100
        tag = 6 + _FEATHER_ID[0]
    P = [m[:3, :3] @ np.array([u, 0.0, v]) + m[:3, 3] for u, v in pts]
    n = m[:3, :3] @ np.array([0, 1.0, 0]); n /= np.linalg.norm(n)
    for k in range(1, len(P) - 1):
        s.tris.append((np.array([P[0], P[k], P[k + 1]]), colour, n, None, {"tag": tag}))

def feather(s, m, length, width, colour):
    """One feather along m's +x from its quill: narrow at the root, widest two thirds out, a rounded tip."""
    pts = [(0, 0)]
    for k in range(9):                                    # one side out to the tip ...
        u = length * k / 8
        pts.append((u, -width * 0.5 * math.sin(math.pi * min(1, (u / length) * 0.85 + 0.12)) ** 0.8))
    for a in np.linspace(-math.pi / 2, math.pi / 2, 5)[1:-1]:          # ... round the tip ...
        pts.append((length + width * 0.35 * math.cos(a), width * 0.45 * math.sin(a)))
    for k in range(8, -1, -1):                            # ... and back down the other side
        u = length * k / 8
        pts.append((u, width * 0.5 * math.sin(math.pi * min(1, (u / length) * 0.85 + 0.12)) ** 0.8))
    flat(s, m, pts, colour)

def wing(s, m, flap):
    """A bird's wing out along m's +x (mirror m for the left one): an arm of coverts with secondaries along its trailing
    edge, hinged to a hand of long primaries that fan out and lag behind the beat."""
    lift = math.radians(65 - 115 * flap)
    arm = m @ rot_z(lift)
    hand = arm @ trans(11, 0, 0.5) @ rot_z(math.radians(-25 + 50 * flap) * 0.6)   # the hand trails the arm's beat
    for k in range(5):                                    # secondaries, pointing back
        feather(s, arm @ trans(1 + k * 2.3, -0.05, 2.5) @ rot_y(-math.radians(80 - k * 4)), 7.5, 4.0, FEATHER_SHADE)
    flat(s, arm @ trans(0, 0.05, 0), [(0, 0), (0, -2.2), (6, -2.6), (11.5, -1.6), (12.5, 2.4), (9, 4.8), (4, 5.4), (0, 4.6)], FEATHER)   # coverts
    for k in range(6):                                    # primaries, fanning out and back from the hand
        feather(s, hand @ trans(0, 0.05 * k, 0.5 + k * 0.5) @ rot_y(-math.radians(5 + k * 12)), 15 - k * 1.3, 4.6, FEATHER if k % 2 == 0 else FEATHER_SHADE)
    flat(s, hand @ trans(0, 0.3, 0), [(0, -1.5), (5, -1.6), (7, 0.2), (4, 2.2), (0, 2.6)], FEATHER)   # the hand's coverts


def mixer(s, flap):
    """A tilt-head stand mixer, unbranded: its nose toward -z, wings on the head flapping at flap (0 up .. 1 down)."""
    s.box(trans(0, 2, 2), (20, 4, 36), RED)                                    # the long base
    s.box(trans(0, 3, -14), (16, 2, 4), RED)                                   # its rounded-looking front lip
    s.box(trans(0, 16, 14), (10, 24, 9), RED)                                  # the column at the back
    head = trans(0, 31, 2)
    ring = [(9 * math.cos(a), 8 * math.sin(a)) for a in np.linspace(0, 2 * math.pi, 16, endpoint=False)]
    prism(s, head @ trans(0, 0, 4), ring, 22, RED)                              # the motor head
    prism(s, head @ trans(0, 0, -11), ring, 8, RED, cap=0.62)                   # tapering to its nose
    prism(s, head @ trans(0, 0, -6.2), [(x * 1.06, y * 1.06) for x, y in ring], 1.6, CHROME)   # the trim band
    prism(s, trans(0, 31, -15.6), circle(2.8, 12), 1.4, CHROME)                # the hub on the nose
    lathe(s, trans(0, 4, -5), [(5.5, 0), (7.5, 1.5), (9.0, 5), (10.0, 10), (10.6, 13)], STEEL)   # the bowl
    s.box(trans(10.8, 9, -5), (1.2, 5, 1.6), STEEL)                            # its handle
    s.box(trans(0, 21, -5), (1.8, 10, 1.8), CHROME)                            # the beater's shaft
    s.box(trans(0, 13, -5), (6, 6, 1.0), CHROME)                               # and its paddle
    s.box(trans(4.5, 25, 12), (1.2, 1.2, 5), DARK)                             # the speed lever
    for side in (-1, 1):                                                        # the wings, from the head's sides
        wing(s, trans(side * 8.5, 35, 3) @ np.diag([side * 1.5, 1.5, 1.5, 1]), flap)   # half as big again

def foodprocessor(s, flap):
    """A generic food processor, its front (the dial) toward -z: a white motor base, a clear bowl with a handle, a lid
    with a feed tube and its pusher; wings from the base's shoulders."""
    WHITE_BODY, PLASTIC, LID, GREY = (330, 330, 324), (220, 236, 248), (236, 246, 255), (70, 72, 80)   # bright, so lighting leaves them white
    ring = [(9 * math.cos(a), 8 * math.sin(a)) for a in np.linspace(0, 2 * math.pi, 16, endpoint=False)]
    s.box(trans(0, 5, 0), (18, 10, 16), WHITE_BODY)                            # the motor base
    prism(s, trans(0, 5, -8.1), circle(2.4, 12), 0.5, GREY)                    # the dial
    for dx in (-5, 5):
        s.box(trans(dx, 4, -8.15), (2.2, 1.4, 0.4), GREY)                      # two buttons
    lathe(s, trans(0, 10, 0), [(7.6, 0), (8.0, 4), (8.2, 9), (8.4, 12)], PLASTIC)   # the clear bowl
    s.box(trans(9.4, 16, 0), (1.6, 7, 2.2), PLASTIC)                           # its handle
    lathe(s, trans(0, 22, 0), [(8.6, 0), (8.6, 0.8), (0.1, 0.9)], LID)        # the lid
    lathe(s, trans(-3, 22.8, 2.5), [(2.6, 0), (2.6, 6.5), (0.1, 6.6)], LID)    # the feed tube
    lathe(s, trans(-3, 29.4, 2.5), [(2.1, 0), (2.1, 1.6), (0.1, 1.7)], WHITE_BODY)   # its pusher
    for side in (-1, 1):
        wing(s, trans(side * 9, 10, 1) @ np.diag([side * 1.5, 1.5, 1.5, 1]), flap)

ISO_DIR = np.array([1.0, 0.0, -1.0]) / math.sqrt(2) * math.cos(math.radians(30)) + np.array([0, -math.sin(math.radians(30)), 0])

def sprite(flap, size=320, model=None, yaw=math.pi, centre_y=18.0):
    """One wing position rendered isometrically: (RGBA image). A far camera with a narrow lens is near enough
    orthographic; the 2:1 pixel-art angle looks down 30 degrees, from 45 round."""
    s = M.Scene()
    (model or mixer)(s, flap)
    R = rot_y(yaw)[:3, :3]
    s.tris = [(p @ R.T, c, None if n is None else R @ n, uv, ex) for p, c, n, uv, ex in s.tris]   # facing the way they fly
    look = ISO_DIR / np.linalg.norm(ISO_DIR)            # down at 30 degrees, from 45 round
    centre = np.array([0, centre_y, 0])
    eye = centre - look * 4000
    basis = M.look(eye, centre)
    hfov = 2 * math.atan(60 / 4000)                         # room for the wings
    img, tone, oid, nrm, izb, tag = M.raster(s.tris, eye, basis, hfov, size, size, {})
    # crisp outlines where the object ends or a face changes
    edge = np.zeros(oid.shape, bool)
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
        o2 = np.roll(oid, (dy, dx), (0, 1)); n2 = np.roll(nrm, (dy, dx), (0, 1))
        edge |= ((o2 != oid) & ((oid == 0) != (o2 == 0))) | ((oid > 0) & (tag < 6) & (np.abs(n2 - nrm).sum(2) > 0.7))
    between = np.zeros(oid.shape, bool)                   # a fine grey line between feathers, pixel-art style
    for dy, dx in ((0, 1), (1, 0)):
        o2, t2 = np.roll(oid, (dy, dx), (0, 1)), np.roll(tag, (dy, dx), (0, 1))
        between |= (t2 != tag) & (tag >= 6) & (t2 >= 6)
    rgba = np.zeros((size, size, 4), np.uint8)
    img = np.where(between[..., None], np.array([150, 152, 168.0]), img)
    rgba[..., :3] = np.where(edge[..., None] & (izb[..., None] > 0), 24, img).astype(np.uint8)
    rgba[..., 3] = (izb > 0) * 255
    out_line = np.zeros(oid.shape, bool)                  # an outline just outside the silhouette too
    sil = izb > 0
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
        out_line |= np.roll(sil, (dy, dx), (0, 1)) & ~sil
    rgba[out_line] = (24, 24, 28, 255)
    return Image.fromarray(rgba)

# the flock, all off the top right when they set off after the pause, streaming in one after another:
# (x, y when they set off, as fractions of the picture, size, speed, phase)
def flock(n=30, flight=9.0, seed=1989):
    """A steady stream: n mixers setting off one after another through the flight, each from just off the right edge
    at its own height, size, speed and wingbeat. (when it sets off, x, y as fractions of the picture, size, speed,
    phase)"""
    rng = np.random.default_rng(seed)
    out = []
    for k in range(n):
        start = max(0.0, (k + rng.uniform(-0.3, 0.3)) * flight / n)   # all come in from off the edge, after the pause
        out.append((start, 1.06 + rng.uniform(0, 0.08), rng.uniform(-0.95, 0.95), rng.uniform(0.28, 0.44),
                    rng.uniform(0.88, 1.06), rng.uniform(0, 1)))
    return out

FLOCK = flock(n=11, flight=3.0)                          # launched over 3 s, so the last is gone before the end

def heading(yaw=math.pi):
    """Which way a model's front (its -z, turned by yaw) points on the picture: a unit vector, y down."""
    look = ISO_DIR / np.linalg.norm(ISO_DIR)
    basis = M.look(-look * 4000, np.zeros(3))
    f = rot_y(yaw)[:3, :3] @ np.array([0, 0, -1.0])
    v = np.array([basis[0] @ f, -(basis[1] @ f)])
    return v / np.linalg.norm(v)

HEADING = heading()
PROC_YAW = min((math.pi / 2 * k for k in range(4)), key=lambda y: np.linalg.norm(heading(y) - np.array([0.894, 0.447])))
PROC_HEADING = heading(PROC_YAW)                         # top left to bottom right, the other isometric diagonal

def exit_time(t0, x0, y0, size, speed, hd, W=768, H=384):
    """When a flier set off at t0 is wholly off the picture."""
    px, t = size * H * 2.0, t0
    while True:
        x = x0 * W + hd[0] * 0.30 * speed * (t - t0) * W
        y = y0 * H + hd[1] * 0.30 * speed * (t - t0) * W
        if x + px * 0.35 < 0 or x - px * 0.35 > W or y - px * 0.35 > H:
            return t
        t += 1 / FPS

def schedule():
    """The mixers set off over 3 s; once the last has gone, the food processors set off over 3 s from the top left;
    then a second of black. (fliers as (kind, t0, x0, y0, size, speed, phase), the flight's length in seconds)"""
    mixers = [("mixer", *f) for f in FLOCK]
    gone = max(exit_time(t0, x0, y0, sz, sp, HEADING) for _, t0, x0, y0, sz, sp, _ in mixers)
    rng = np.random.default_rng(1993)
    procs = []
    for k, (t0, x0, y0, size, speed, ph) in enumerate(flock(n=11, flight=3.0, seed=1993)):
        procs.append(("proc", gone + 0.3 + t0, -0.06 - rng.uniform(0, 0.08), y0, size, speed, ph))
    end = max(exit_time(t0, x0, y0, sz, sp, PROC_HEADING) for _, t0, x0, y0, sz, sp, _ in procs)
    return mixers + procs, end + 1.6                       # a full second of black after the last has really gone

FLIERS, FLIGHT = schedule()
FRAMES = PAUSE + int(math.ceil(FLIGHT * FPS))

def main():
    out = sys.argv[1]
    W, H = map(int, (sys.argv[2] if len(sys.argv) > 2 else "192x96").split("x"))
    looks = {"mixer": ([sprite(0.5 - 0.5 * math.cos(2 * math.pi * k / PHASES)) for k in range(PHASES)], HEADING),
             "proc": ([sprite(0.5 - 0.5 * math.cos(2 * math.pi * k / PHASES), model=foodprocessor, yaw=PROC_YAW, centre_y=15.0)
                       for k in range(PHASES)], PROC_HEADING)}
    with open(out, "wb") as fh:
        for i in range(FRAMES):
            im = Image.new("RGB", (W, H))
            t = (i - PAUSE) / FPS
            for kind, t0, x0, y0, size, speed, ph in sorted(FLIERS, key=lambda f: f[4]):
                if t < t0:
                    continue
                sprites, hd = looks[kind]
                px = int(size * H * 2.0)                  # the sprites have room round them (for the wings)
                sp = sprites[int((t * 2.2 + ph) * PHASES) % PHASES].resize((px, px), Image.NEAREST if px < 120 else Image.BILINEAR)
                x = x0 * W + hd[0] * 0.30 * speed * (t - t0) * W - px / 2   # along where it faces
                y = y0 * H + hd[1] * 0.30 * speed * (t - t0) * W - px / 2
                im.paste(sp, (int(x), int(y)), sp)
            fh.write(np.asarray(im).tobytes())

if __name__ == "__main__":
    main()
