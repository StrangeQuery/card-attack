"""Prototype: the "Money for Nothing" shop scene (Dire Straits video, 1:31-1:35), the two movers walking past a big
TV, with an earlier Card Attack segment playing on the screen. Flat-shaded boxes, z-buffered, the screen texture
mapped with perspective. 48 frames at 12 fps.

    python mtv.py SCREEN OUT.rgb [WxH]

SCREEN is plasma, roto, pump (the flash's pumping CARDIAC keyed over the plasma), neon (that CARDIAC in the
guitar shot's neon look, over the plasma) or neonblack (the same on black), read from
this repository's segments/. Writes raw RGB frames.
"""
import math
import os
import sys
import pathlib

import numpy as np


def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


SEG = pathlib.Path(__file__).resolve().parent.parent / "segments"
SCREENS = {"plasma": (SEG / "plasma/video.rgb", 0), "roto": (SEG / "roto/video4.rgb", 60)}   # file, first frame (after the roto's opening flash)
FRAMES = 48

# ---- geometry: boxes in a hierarchy ------------------------------------------------------------------------------

def rot_x(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[1, 0, 0, 0], [0, c, -s, 0], [0, s, c, 0], [0, 0, 0, 1]])

def rot_y(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, 0, s, 0], [0, 1, 0, 0], [-s, 0, c, 0], [0, 0, 0, 1]])

def rot_z(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, -s, 0, 0], [s, c, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]])

def trans(x, y, z):
    m = np.eye(4); m[:3, 3] = (x, y, z); return m

CUBE = np.array([[x, y, z] for x in (-.5, .5) for y in (-.5, .5) for z in (-.5, .5)])
# faces as corner indices (counter-clockwise seen from outside), with their outward normals
FACES = [((0, 1, 3, 2), (-1, 0, 0)), ((4, 6, 7, 5), (1, 0, 0)), ((0, 4, 5, 1), (0, -1, 0)),
         ((2, 3, 7, 6), (0, 1, 0)), ((0, 2, 6, 4), (0, 0, -1)), ((1, 5, 7, 3), (0, 0, 1))]


class Scene:
    def __init__(self):
        self.tris = []      # (3x3 world points, rgb, normal or None for unlit, uv or None)

    def box(self, m, size, rgb, faces=None):
        """A box of size (w, h, d) centred at m's origin; faces: per-face colours by name, else rgb."""
        pts = (CUBE * size) @ m[:3, :3].T + m[:3, 3]
        names = ("left", "right", "bottom", "top", "front", "back")
        for (idx, n), name in zip(FACES, names):
            col = faces.get(name, rgb) if faces else rgb
            normal = m[:3, :3] @ np.array(n, float)
            normal /= np.linalg.norm(normal)
            q = pts[list(idx)]
            self.tris.append((q[[0, 1, 2]], col, normal, None))
            self.tris.append((q[[0, 2, 3]], col, normal, None))

    def quad(self, m, w, h, rgb, z=0.0, uv=False, lit=True):
        """A rectangle w x h in m's xy plane at depth z, facing -z (toward a camera in front)."""
        p = np.array([[-w / 2, -h / 2, z], [-w / 2, h / 2, z], [w / 2, h / 2, z], [w / 2, -h / 2, z]])
        pts = p @ m[:3, :3].T + m[:3, 3]
        normal = m[:3, :3] @ np.array([0, 0, -1.0])
        t = np.array([[0, 1], [0, 0], [1, 0], [1, 1]], float)     # u right, v down
        for a, b, c in ((0, 1, 2), (0, 2, 3)):
            self.tris.append((pts[[a, b, c]], rgb, normal if lit else None, t[[a, b, c]] if uv else None))

# ---- the set -----------------------------------------------------------------------------------------------------

WALL = (138, 104, 72)
PILLAR = (58, 82, 140)
BEAM = (246, 246, 250)
FLOOR = (104, 104, 110)
SKIRT = (176, 176, 180)
TVFRAME = (18, 18, 22)

def shop(s):
    s.box(trans(0, 225, 720), (1600, 450, 20), WALL)
    for x in (-560, -330, -100, 420):
        s.box(trans(x, 225, 706), (22, 450, 12), PILLAR)
    s.box(trans(0, 6, 704), (1600, 12, 8), SKIRT)
    s.box(trans(0, -5, 380), (1600, 10, 680), FLOOR)
    for x in (-430, -170):                                # strip lights running back along the ceiling
        s.box(trans(x, 262, 470), (34, 12, 500), BEAM)
        s.box(trans(x, 252, 470), (20, 8, 500), (250, 250, 250))
    # the TV: a black frame, turned a little away on its right and tipped forward at the top
    tv = trans(150, 230, 540) @ rot_y(math.radians(24)) @ rot_x(math.radians(-12))
    s.box(tv @ trans(0, 0, 8), (420, 216, 16), TVFRAME)
    s.quad(tv, 400, 200, (0, 0, 0), z=-0.6, uv=True, lit=False)
    s.box(tv @ trans(0, 120, 0), (10, 30, 10), TVFRAME)        # its hanger

# ---- the movers ---------------------------------------------------------------------------------------------------

def walk(phase, amp):
    return amp * math.sin(phase), amp * math.sin(phase + math.pi)

def thin(s, m, phase):
    """The tall thin one: red cap, striped shirt, lilac straps, microwave held in front."""
    SKIN, HAIR, CAP = (176, 110, 82), (52, 30, 24), (206, 36, 36)
    LEG, STRIPE_A, STRIPE_B = (156, 116, 140), (236, 130, 206), (244, 236, 240)
    GLOVE = (236, 164, 40)
    bob = 2.5 * abs(math.sin(phase))
    m = m @ trans(0, bob, 0)
    a, b = walk(phase, math.radians(24))
    for side, swing in ((-1, a), (1, b)):                  # legs: thigh and shin from the hip
        hip = m @ trans(side * 9, 92, 0) @ rot_x(swing)
        s.box(hip @ trans(0, -24, 0), (13, 48, 13), LEG)
        knee = hip @ trans(0, -48, 0) @ rot_x(-max(0.0, swing) * 0.8)
        s.box(knee @ trans(0, -22, 0), (11, 44, 11), LEG)
        s.box(knee @ trans(0, -46, -5), (12, 6, 22), (60, 40, 50))
    torso = m @ trans(0, 92, 0)
    for k in range(6):                                     # the shirt in stripes
        s.box(torso @ trans(0, 6 + k * 9, 0), (30, 9, 16), STRIPE_A if k % 2 == 0 else STRIPE_B)
    for side in (-1, 1):                                   # overall straps
        s.box(torso @ trans(side * 8, 32, -8.5), (5, 48, 1), (190, 160, 220))
    s.box(torso @ trans(0, 60, 0), (8, 6, 8), SKIN)        # neck
    head = torso @ trans(0, 76, 0) @ rot_y(math.radians(-40))
    s.box(head, (16, 28, 18), SKIN, {"back": HAIR})
    s.box(head @ trans(-8.3, 6, 6), (0.6, 8, 5), HAIR)                 # hair over the ear
    s.box(head @ trans(0, 2, -9.3), (14, 3, 0.6), (40, 24, 20))        # brows
    s.box(head @ trans(0, -4, -10), (5, 8, 3), (160, 96, 70))          # nose
    s.box(head @ trans(0, -10, -9.3), (8, 2, 0.6), (120, 50, 40))      # mouth
    s.box(head @ trans(0, 16, 1), (18, 7, 20), CAP)
    s.box(head @ trans(0, 13.5, -13), (16, 2, 10), CAP)                # peak
    for side in (-1, 1):                                   # arms down, forearms forward to the box
        sh = torso @ trans(side * 18, 52, 0)
        s.box(sh @ trans(0, -16, 0), (8, 32, 8), STRIPE_B)
        s.box(sh @ trans(0, -32, -12), (7, 7, 24), SKIN)
        s.box(sh @ trans(-side * 1, -32, -24), (9, 9, 9), GLOVE)
    microwave(s, torso @ trans(0, 16, -38) @ rot_x(math.radians(4)), scale=1.15)

def stocky(s, m, phase):
    """The stocky one: blue cap, beard, glasses, teal shirt, blue overalls."""
    SKIN, BEARD, CAP = (214, 140, 108), (74, 40, 30), (30, 62, 204)
    SHIRT, OVERALL, GLOVE = (150, 222, 200), (40, 64, 176), (236, 164, 40)
    bob = 2.0 * abs(math.sin(phase))
    m = m @ trans(0, bob, 0)
    a, b = walk(phase, math.radians(20))
    for side, swing in ((-1, a), (1, b)):
        hip = m @ trans(side * 12, 76, 0) @ rot_x(swing)
        s.box(hip @ trans(0, -20, 0), (18, 40, 18), OVERALL)
        knee = hip @ trans(0, -40, 0) @ rot_x(-max(0.0, swing) * 0.8)
        s.box(knee @ trans(0, -18, 0), (16, 36, 16), OVERALL)
        s.box(knee @ trans(0, -38, -5), (16, 7, 26), (50, 36, 30))
    torso = m @ trans(0, 76, 0)
    s.box(torso @ trans(0, 26, 0), (52, 52, 30), SHIRT)
    s.box(torso @ trans(0, 14, -15.4), (40, 28, 1), OVERALL)          # bib
    for side in (-1, 1):
        s.box(torso @ trans(side * 14, 40, -15.4), (6, 24, 1), OVERALL)
        s.box(torso @ trans(side * 14, 28, -16.2), (4, 4, 1), (240, 210, 40))   # buttons
    head = torso @ trans(0, 68, 0) @ rot_y(math.radians(-15))
    s.box(head, (28, 30, 26), SKIN)
    s.box(head @ trans(0, -9, -6), (29, 14, 16), BEARD)                # beard and jaw
    s.box(head @ trans(0, -6, -13.6), (14, 4, 1), (240, 240, 236))    # teeth
    for side in (-1, 1):                                   # glasses
        s.box(head @ trans(side * 6.5, 4, -13.4), (10, 7, 0.8), (236, 236, 236))
        s.box(head @ trans(side * 6.5, 4, -13.9), (6, 4, 0.8), (40, 30, 30))
    s.box(head @ trans(0, -1, -15), (6, 7, 6), (196, 120, 90))         # nose
    s.box(head @ trans(0, 17, 1), (30, 9, 28), CAP)
    s.box(head @ trans(0, 14, -17), (24, 2, 12), CAP)
    for side in (-1, 1):
        sh = torso @ trans(side * 30, 44, 0)
        s.box(sh @ trans(0, -12, 0), (14, 26, 14), SHIRT, {"left": (100, 170, 150), "right": (100, 170, 150)})
        s.box(sh @ trans(0, -26, -14), (12, 12, 28), SKIN)
        s.box(sh @ trans(-side * 2, -26, -30), (13, 13, 12), GLOVE)
    microwave(s, torso @ trans(0, 18, -44) @ rot_x(math.radians(3)), scale=1.15)

def microwave(s, m, scale=1.0):
    w, h, d = 50 * scale, 32 * scale, 34 * scale
    SIDE, CREAM, WINDOW = (124, 62, 30), (242, 204, 164), (112, 22, 22)
    s.box(m, (w, h, d), SIDE, {"front": CREAM, "top": (150, 80, 40)})
    s.box(m @ trans(-w * 0.12, 0, -d / 2 - 0.4), (w * 0.56, h * 0.62, 0.6), WINDOW)
    s.box(m @ trans(w * 0.36, 0, -d / 2 - 0.4), (w * 0.12, h * 0.8, 0.6), (40, 150, 64))
    for k in range(4):
        s.box(m @ trans(w * 0.36, h * (0.28 - k * 0.18), -d / 2 - 0.8), (w * 0.06, h * 0.07, 0.6), (240, 220, 40))

# ---- the camera and the rasteriser -------------------------------------------------------------------------------

LIGHT = np.array([-0.45, 0.75, -0.5]); LIGHT /= np.linalg.norm(LIGHT)
CAM = (np.array([0.0, 115.0, 0.0]), math.radians(-8))     # position, pitch down (negative: looking up)
HFOV = math.radians(76)

def render(scene, W, H, screen):
    pos, pitch = CAM
    view = rot_x(-pitch)[:3, :3]                          # world -> camera: undo the pitch
    f = (W / 2) / math.tan(HFOV / 2)
    img = np.zeros((H, W, 3), np.float32)
    zbuf = np.zeros((H, W), np.float32)                   # 1/z: bigger is nearer
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32) + 0.5
    for pts, rgb, normal, uv in scene.tris:
        c = (pts - pos) @ view.T
        if (c[:, 2] < 5).any():
            continue
        sx = W / 2 + f * c[:, 0] / c[:, 2]
        sy = H / 2 - f * c[:, 1] / c[:, 2]
        x0, x1 = int(max(0, math.floor(sx.min()))), int(min(W, math.ceil(sx.max()) + 1))
        y0, y1 = int(max(0, math.floor(sy.min()))), int(min(H, math.ceil(sy.max()) + 1))
        if x0 >= x1 or y0 >= y1:
            continue
        px, py = xs[y0:y1, x0:x1], ys[y0:y1, x0:x1]
        d = (sx[1] - sx[0]) * (sy[2] - sy[0]) - (sx[2] - sx[0]) * (sy[1] - sy[0])
        if abs(d) < 1e-9:
            continue
        l1 = ((px - sx[0]) * (sy[2] - sy[0]) - (sx[2] - sx[0]) * (py - sy[0])) / d
        l2 = ((sx[1] - sx[0]) * (py - sy[0]) - (px - sx[0]) * (sy[1] - sy[0])) / d
        l0 = 1 - l1 - l2
        inside = (l0 >= -1e-4) & (l1 >= -1e-4) & (l2 >= -1e-4)
        iz = l0 / c[0, 2] + l1 / c[1, 2] + l2 / c[2, 2]
        zb = zbuf[y0:y1, x0:x1]
        win = inside & (iz > zb)
        if not win.any():
            continue
        zb[win] = iz[win]
        if uv is not None:                                # the screen: perspective-correct texture lookup
            u = (l0 * uv[0, 0] / c[0, 2] + l1 * uv[1, 0] / c[1, 2] + l2 * uv[2, 0] / c[2, 2]) / iz
            v = (l0 * uv[0, 1] / c[0, 2] + l1 * uv[1, 1] / c[1, 2] + l2 * uv[2, 1] / c[2, 2]) / iz
            th, tw = screen.shape[:2]
            tu = np.clip((u * tw).astype(int), 0, tw - 1)
            tv = np.clip((v * th).astype(int), 0, th - 1)
            img[y0:y1, x0:x1][win] = screen[tv[win], tu[win]]
        else:
            k = 1.0 if normal is None else 0.6 + 0.4 * max(0.0, float(normal @ LIGHT)) + 0.08 * max(0.0, float(-normal[2]))
            img[y0:y1, x0:x1][win] = np.minimum(255, np.array(rgb, np.float32) * k)
    return img.astype(np.uint8)

# ---- the shot ----------------------------------------------------------------------------------------------------

def shot(i):
    """Frame i of 48: the thin one crosses behind (frames 0-34), the stocky one close in front (from 12)."""
    s = Scene()
    shop(s)
    tx = -200 + 15 * i                                  # cm a frame
    thin(s, trans(tx, 0, 300) @ rot_y(math.radians(-35)) @ np.diag([1.12, 1.12, 1.12, 1]), i * 0.52)
    if i >= 12:
        k = i - 12
        stocky(s, trans(-200 + 11 * k, 0, 195) @ rot_y(math.radians(-25)), k * 0.48)
    return s

# The flash segment's pumping CARDIAC, lit red and blue in turn, alone on a green key (cardlayer.rs, not in this
# repository, renders it to cardlayer.rgb in $CARD_ATTACK_WORK).
CARDLAYER = "cardlayer.rgb"

def neon(c, card, rng, strokes=True):
    """The guitar shot's look (the video, 0:34-0:37) on the card: a navy duotone body, the near edge and the tabs in
    flat magenta, a soft mustard glow round the far side, and cyan strokes bristling off the near side, new each frame."""
    H, W = card.shape
    out = np.zeros((H, W, 3), np.float32)
    lum = c.max(axis=2) / 255.0
    navy_dark, navy_light = np.array([10, 12, 64.0]), np.array([96, 108, 200.0])
    body = navy_dark + (navy_light - navy_dark) * (lum[..., None] ** 2.2)
    # magenta: within 4 texels of the card's left edge on each row, and anything in a narrow run (the tabs)
    magenta = np.zeros_like(card)
    for y in range(H):
        xs = np.flatnonzero(card[y])
        if len(xs) == 0:
            continue
        runs = np.split(xs, np.flatnonzero(np.diff(xs) > 1) + 1)
        for r in runs:
            if len(r) <= 6:
                magenta[y, r] = True
        magenta[y, xs[(xs - xs[0]) < 9]] = True
    # glow: the silhouette pushed down and right, softened, outside the card
    glow = np.zeros((H, W), np.float32)
    for dy in range(-2, 7):
        for dx in range(-2, 8):
            w = max(0.0, 1.0 - math.hypot(dx - 2.5, dy - 2) / 6.0) ** 0.7
            glow = np.maximum(glow, w * np.roll(np.roll(card, dy, 0), dx, 1))
    glow[card] = 0
    out += glow[..., None] * np.array([214, 176, 10.0])
    out[card] = body[card]
    out[magenta & card] = (236, 36, 226)
    # cyan strokes: from points on the card's left and top edges, outward and a little up, each its own length
    edge = card & ~np.roll(card, 1, 1)
    ys, xs = np.nonzero(edge) if strokes else ((), ())
    for y, x in zip(ys, xs):
        if rng.random() < 0.55:
            continue
        n = rng.integers(10, 28)
        ang = math.radians(rng.uniform(150, 215))                 # pointing left, fanned
        for t in range(1, n):
            px, py = int(round(x + t * math.cos(ang))), int(round(y - t * math.sin(ang) - t * 0.3))
            if 0 <= px < W and 0 <= py < H and not card[py, px]:
                out[py, px] = (70, 214, 176)
    return np.clip(out, 0, 255).astype(np.uint8), card | (glow > 0.05) | (out.sum(axis=2) > 0)

def card_over_plasma(style=None, background="plasma"):
    plasma = np.fromfile(SEG / "plasma/video.rgb", np.uint8).reshape(-1, 96, 192, 3)
    layer = np.fromfile(_work(CARDLAYER), np.uint8).reshape(-1, 96, 192, 3)
    out = []
    rng = np.random.default_rng(1985)
    for i in range(FRAMES):
        c = layer[i % len(layer)].astype(int)
        card = ~((c[..., 1] > 200) & (c[..., 0] < 60) & (c[..., 2] < 60))     # everything but the green key
        f = plasma[i % len(plasma)].copy() if background == "plasma" else np.zeros_like(plasma[0])
        if style == "neon":
            look, cover = neon(c, card, rng)
            f[cover] = look[cover]
        else:
            f[card] = c[card]
        out.append(f)
    return np.array(out, np.uint8)

def main():
    which, out = sys.argv[1], sys.argv[2]
    W, H = map(int, (sys.argv[3] if len(sys.argv) > 3 else "192x96").split("x"))
    if which in ("pump", "neon", "neonblack"):           # the pumping CARDIAC keyed over the plasma, in front like the band
        video = card_over_plasma("neon" if which != "pump" else None, "black" if which == "neonblack" else "plasma")
        first = 0
    else:
        path, first = SCREENS[which]
        video = np.fromfile(path, np.uint8).reshape(-1, 96, 192, 3)
    with open(out, "wb") as fh:
        for i in range(FRAMES):
            fh.write(render(shot(i), W, H, video[(first + i) % len(video)]).tobytes())

if __name__ == "__main__":
    main()
