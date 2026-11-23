"""Prototype: Money for Nothing into Take On Me (a-ha video 1:36-1:43).

The shop (mtv.py's 48 frames); then, as the camera pushes in, the TV comes off the wall and stands upright like the
a-ha frame, and the shop turns to pencil, while through the screen is the real world, lit lavender, with the pumping
CARDIAC (the girl). The Amiga and its Boing ball (the boy) materialise on our side, drawn. Then the camera swings
round the frame as in the video: whatever is on the camera's side of the frame is pencil, whatever is seen through the
opening is real, in colour. By the end the CARDIAC is drawn and the Amiga is in colour.

    python mtv_aha.py OUT.rgb [WxH] [FIRST:COUNT]

One scene, one rule (proto turntable.py's, which is the video's): a pixel is colour when its line of sight passes
through the opening and meets something beyond the frame's plane; everything else is pencil.
"""
import json
import math
import pathlib
import sys

import numpy as np

import mtv
from mtv import trans, rot_x, rot_y, rot_z
from aha_camera import rotation

HERE = pathlib.Path(__file__).resolve().parent
EMPTY = 8                                               # the shop empty, before the movers come through
SHOP = 64                                               # Money for Nothing: empty, the movers cross and go
FADE, MOVE, ENTER = 12, 30, 36                          # the transition: fade to pencil, the TV to the portal, the Amiga in
ORBIT, HOLD = 85, 39                                    # Take On Me, and the closing hold (the ball drops in it)
O0 = SHOP + FADE + MOVE + ENTER                         # the orbit's first frame
FRAMES = O0 + ORBIT + HOLD
TVW, TVH = 300.0, 225.0                                 # the opening: a 4:3 set, like the video's
CENTRE = np.array([150.0, 190.0, 540.0])                # where the frame stands, upright, once off the wall
AHA = json.loads((pathlib.Path(__file__).resolve().parent / "aha_camera.json").read_text())   # aha_camera.py's fit
S = 260.0                                               # the opening's width once free (square, as measured)
BORDER, THICK = AHA["border"] * S, AHA["thickness"] * S
PAPER, INK = np.array([176, 172, 196.0]), np.array([27, 37, 43.0])
LAVENDER = np.array([206, 192, 236.0])
FACE = np.fromfile(HERE / "face.rgb", np.uint8).reshape(301, 340, 3)
def boing_screen(w=128, h=96):
    """The Boing demo's room on the monitor: light grey, a magenta grid on the back wall, the floor's in perspective."""
    img = np.full((h, w, 3), 176.0)
    MAG = (166, 0, 166)
    horizon = int(h * 0.72)
    for x in range(8, w - 7, 12):
        img[6:horizon, x] = MAG
    for y in range(6, horizon + 1, 12):
        img[y, 8:w - 7] = MAG
    for k in range(-7, 8):                                 # floor lines toward the middle of the wall
        for y in range(horizon, h):
            x = int(w / 2 + k * 12 * (1 + 2.2 * (y - horizon) / (h - horizon)))
            if 0 <= x < w:
                img[y, x] = MAG
    for y in (horizon + 4, horizon + 10, horizon + 19):
        img[y] = MAG
    return img, img.mean(2), None

BOING = boing_screen()
STRIPS = [(50, 53, 16), (103, 59, 12), (157, 16, 55), (214, 22, -8), (280, -12, -12)]   # texel x, top, bottom tab

def ease(x):
    x = min(1.0, max(0.0, x))
    return x * x * (3 - 2 * x)

def lerp(a, b, t):
    return a + (b - a) * t

# ---- scene: triangles with a dict of extras (pencil tone, tag, texture) ------------------------------------------

CUBE = mtv.CUBE
FACES = mtv.FACES
NAMES = ("left", "right", "bottom", "top", "front", "back")

class Scene:
    def __init__(self):
        self.tris = []

    def box(self, m, size, rgb, tone=None, tag=0, faces=None, side_tone=None, face_tones=None):
        pts = (CUBE * size) @ m[:3, :3].T + m[:3, 3]
        for (idx, n), name in zip(FACES, NAMES):
            col = faces.get(name, rgb) if faces else rgb
            normal = m[:3, :3] @ np.array(n, float); normal /= np.linalg.norm(normal)
            t = side_tone if (side_tone is not None and name not in ("front", "back")) else tone
            if face_tones and name in face_tones:
                t = face_tones[name]
            ex = {"tag": tag, "tone": t}
            q = pts[list(idx)]
            self.tris.append((q[[0, 1, 2]], col, normal, None, ex))
            self.tris.append((q[[0, 2, 3]], col, normal, None, ex))

    def ring(self, m, w, h, hw, hh, d, rgb, tone, edge_tone, tag=0):
        """A frame round an opening hw x hh: its outer edges dark (the video's black edge), its inner ones light."""
        parts = ((0, (h + hh) / 4, w, (h - hh) / 2, "top"), (0, -(h + hh) / 4, w, (h - hh) / 2, "bottom"),
                 ((w + hw) / 4, 0, (w - hw) / 2, hh, "right"), (-(w + hw) / 4, 0, (w - hw) / 2, hh, "left"))
        for cx, cy, sw, sh, outer in parts:
            ft = {n: tone for n in ("left", "right", "top", "bottom")}
            ft[outer] = edge_tone
            if outer in ("top", "bottom"):
                ft["left"] = ft["right"] = edge_tone
            self.box(m @ trans(cx, cy, 0), (sw, sh, d), rgb, tone, tag, face_tones=ft)

    def quad(self, m, w, h, tex, tag=0, cutout=False, u=(0.0, 1.0)):
        p = np.array([[-w / 2, -h / 2, 0], [-w / 2, h / 2, 0], [w / 2, h / 2, 0], [w / 2, -h / 2, 0]])
        pts = p @ m[:3, :3].T + m[:3, 3]
        normal = m[:3, :3] @ np.array([0, 0, -1.0])
        uv = np.array([[u[0], 1], [u[0], 0], [u[1], 0], [u[1], 1]], float)
        ex = {"tag": tag, "tex": tex, "cutout": cutout}
        for a, b, c in ((0, 1, 2), (0, 2, 3)):
            self.tris.append((pts[[a, b, c]], (0, 0, 0), normal, uv[[a, b, c]], ex))

# ---- the places -----------------------------------------------------------------------------------------------------

def shop(s, i, tvm):
    """mtv.py's shop in colour: its room, the TV and its picture, the movers while they cross."""
    shop_room(s)
    frame(s, tvm, 0.0)
    s.quad(tvm @ trans(0, 0, -0.6), TVW, TVH, "tvpic", u=(1 / 6, 5 / 6))      # the middle 4:3 of the picture
    mtv_movers(s, i)

def shop_room(s):
    """The shop: brown wall right behind the TV, pillars, floor, strip lights."""
    s.box(trans(0, 225, 720), (1600, 450, 20), mtv.WALL, 225, 2)
    for x in (-560, -330, -100, 420):
        s.box(trans(x, 225, 706), (22, 450, 12), mtv.PILLAR, 200, 2)
    s.box(trans(0, 6, 704), (1600, 12, 8), mtv.SKIRT, 225, 2)
    s.box(trans(0, -5, 300), (1600, 10, 1200), mtv.FLOOR, 225, 2)
    for x in (-430, -170):
        s.box(trans(x, 262, 470), (34, 12, 500), mtv.BEAM, 230, 2)

class _Adapter:
    """Lets mtv.py's movers (which call box(m, size, rgb, faces)) build into this Scene."""
    def __init__(self, s):
        self.s = s
    def box(self, m, size, rgb, faces=None):
        tone = 235 if np.mean(rgb) > 150 else (150 if np.mean(rgb) > 90 else 70)
        self.s.box(m, size, rgb, tone, 0, faces)

def mtv_movers(s, i):
    """mtv.py's two movers, coming in from off the left after EMPTY frames and gone off the right before SHOP."""
    a = _Adapter(s)
    k = i - EMPTY
    if 0 <= k < 42:
        mtv.thin(a, trans(-270 + 15 * k, 0, 300) @ rot_y(math.radians(-35)) @ np.diag([1.12, 1.12, 1.12, 1]), k * 0.52)
    k = i - EMPTY - 14
    if 0 <= k < 42:
        mtv.stocky(a, trans(-230 + 11 * k, 0, 195) @ rot_y(math.radians(-25)), k * 0.48)

def studio_room(s):
    """One space round the free-standing frame. The near half is the shop (brown, grey floor), the far half the real
    world of the video (lavender, black panel rectangles on the walls). Seen through the frame it's in colour;
    everywhere else it's drawn."""
    z0 = CENTRE[2]
    NEAR_FLOOR = FAR_FLOOR = LAV = WALL = PILLAR = (0, 0, 0)   # in colour the room is black: the characters on black
    s.box(trans(100, -5, z0 - 500), (2200, 10, 1000), NEAR_FLOOR, 228, 2)
    s.box(trans(100, -5, z0 + 500), (2200, 10, 1000), FAR_FLOOR, 228, 2)
    s.box(trans(100, 300, z0 + 1000), (2200, 600, 20), LAV, 228, 2)               # far wall
    s.box(trans(100, 300, z0 - 1000), (2200, 600, 20), WALL, 228, 2)              # near wall, behind us
    for x, col in ((-1000, None), (1200, None)):
        s.box(trans(x, 300, z0 + 500), (20, 600, 1000), LAV, 228, 2)
        s.box(trans(x, 300, z0 - 500), (20, 600, 1000), WALL, 228, 2)
    # the video's drawn panels: black rectangles on the far walls (outlines), and pillars on the near ones
    rng = np.random.default_rng(3)
    for k in range(6):
        w, h = rng.uniform(120, 300), rng.uniform(80, 200)
        x, y = -800 + k * 340 + rng.uniform(-30, 30), rng.uniform(120, 420)
        panel_outline(s, trans(x, y, z0 + 988), w, h)
    for k in range(3):
        w, h = rng.uniform(120, 260), rng.uniform(80, 200)
        z, y = z0 + 160 + k * 260, rng.uniform(120, 420)
        panel_outline(s, trans(-988, y, z) @ rot_y(math.radians(90)), w, h)
        panel_outline(s, trans(1188, y, z + 60) @ rot_y(math.radians(-90)), w, h)
    for x in (-700, -300, 100, 500, 900):
        s.box(trans(x, 300, z0 - 988), (24, 600, 10), PILLAR, 228, 2)

def panel_outline(s, m, w, h, t=4):
    for cx, cy, sw, sh in ((0, h / 2, w, t), (0, -h / 2, w, t), (w / 2, 0, t, h), (-w / 2, 0, t, h)):
        s.box(m @ trans(cx, cy, 0), (sw, sh, 1.5), (0, 0, 0), 200, 2)

def frame(s, tvm, off):
    """The TV, now a free-standing frame: drawn like the video's (a white board with a dark edge); black in colour."""
    w, h = tv_size(off)
    s.ring(tvm @ trans(0, 0, THICK / 2), w + 2 * BORDER, h + 2 * BORDER, w, h, THICK, mtv.TVFRAME, 232, 30)

# Where the girl and the boy are across the video's picture (x of their middles, 960 wide) in the measured keyframes
GIRL_PX = [275, 320, 360, 250, 250, 160, 70]     # ending further left than the video's girl
BOY_PX = [760, 790, 800, 680, 580, 520, 550]      # further right than the video's boy early on: the portal clear
GIRL_YZ = [(CENTRE[1] + 0.08 * S, CENTRE[2] + z * S)                  # she stays beyond the frame, drifting
           for z in (0.35, 0.35, 0.35, 0.38, 0.42, 0.48, 0.52)]        # back a little to leave the portal clear
BOY_YZ = [(CENTRE[1] - 0.12 * S, CENTRE[2] - z * S)                     # he starts just this side of it and backs
          for z in (0.30, 0.32, 0.38, 0.48, 0.64, 0.86, 1.00)]         # well away, to be seen whole through it at the end

def solve_x(k, yz, px):
    """The x that puts a point at height y, depth z at picture x px through keyframe k's camera (exactly: the
    camera-space coordinates are linear in x, so the picture x is a ratio of two linear terms)."""
    eye, basis, _ = aha_camera(AHA["keys"][k])
    A = basis @ (np.array([0.0, yz[0], yz[1]]) - eye)
    c = basis[:, 0]
    u = (px - 480) / AHA["f"]
    return (u * A[2] - A[0]) / (c[0] - u * c[2])

_XS = {}

def xs(who):
    """The solved x positions at the keyframes (worked out once, the camera code being further down)."""
    if who not in _XS:
        yz, pxs = (GIRL_YZ, GIRL_PX) if who == "girl" else (BOY_YZ, BOY_PX)
        x = [solve_x(k, yz[k], px) for k, px in enumerate(pxs)]
        # edge-on the camera is nearly level with them, so a picture position pins nothing: take the neighbours'
        med = float(np.median(x))
        for k in range(len(x)):
            if abs(x[k] - med) > 1.5 * S or (who == "boy" and k == 3):
                # edge-on (and, for him, just before it, where his picture position would put him on the camera)
                # a picture position pins nothing useful: take the neighbours'
                x[k] = (x[max(0, k - 1)] + x[min(len(x) - 1, k + 1)]) / 2
        _XS[who] = x
    return _XS[who]

def places(t):
    """Where the CARDIAC (the girl) and the Amiga (the boy) are at orbit frame t: placed so that through the measured
    camera they sit where the girl and the boy sit in the video. Neither crosses the frame."""
    t = 0.0 if t is None else t
    keys = np.array(AHA["keys"], float)
    gy, gz = (catmull(keys, np.array([v[k] for v in GIRL_YZ]), t) for k in (0, 1))
    by, bz = (catmull(keys, np.array([v[k] for v in BOY_YZ]), t) for k in (0, 1))
    g = np.array([catmull(keys, np.array(xs("girl")), t), gy, gz])
    b = np.array([catmull(keys, np.array(xs("boy")), t), by, bz])
    return g, b

def facing(p, q):
    """The turn about the vertical that points a front facing -z from p toward q."""
    d = q - p
    return math.atan2(-d[0], -d[2])

def cardiac(s, i, tvm, off):
    """The girl: the Money for Nothing CARDIAC (the flash's pumping card in the guitar shot's neon), cut out of its
    black. Until the Amiga comes in she sits just behind the screen exactly where the TV's picture showed her (so the
    picture becoming the portal shows no change); while it comes in she drifts back to her place in the shot, and from
    then on she floats beyond the frame, facing the Amiga."""
    w, h = tv_size(off)
    u = ease((i - (O0 - ENTER)) / ENTER)
    if u <= 0:
        s.quad(tvm @ trans(0, 0, 3), 2 * h, h, "neon", tag=4, cutout=True)    # 2:1, as the picture was
        return
    g, b = places(max(0, i - O0))
    pos = lerp(tvm[:3, 3] + np.array([0, 0, 3.0]), g, u)
    s.quad(trans(*pos) @ rot_y(u * facing(g, b)), lerp(2 * h, 320, u), lerp(h, 160, u), "neon", tag=4, cutout=True)

AMIGA_IN = 72                                           # frames the Amiga takes to come in, arriving as the orbit starts
HESITATE = [(0.0, 0.0), (0.18, 0.30), (0.30, 0.30), (0.40, 0.45), (0.46, 0.41), (0.60, 0.41), (0.82, 0.85), (1.0, 1.0)]
AMIGA_CLOSER = 72                                       # frames of the orbit over which it edges the last 20% closer
HESITATE_CLOSER = [(0.0, 0.0), (0.22, 0.35), (0.36, 0.35), (0.52, 0.62), (0.60, 0.56), (0.74, 0.56), (1.0, 1.0)]

def hesitant(x, steps):
    """How far in the Amiga is at time x (0..1) of its entrance: in a little and stop, edge forward, shy back a touch,
    wait, then the rest of the way. Each stretch eased, so it starts and stops softly."""
    if x <= 0:
        return 0.0
    for (x0, y0), (x1, y1) in zip(steps, steps[1:]):
        if x <= x1:
            return y0 + (y1 - y0) * ease((x - x0) / (x1 - x0))
    return 1.0

def amiga(s, i):
    """The boy: an A500 with its monitor, floating on our side, the Boing ball bouncing in front of the screen. He
    comes in from the right while the camera holds, then keeps his place in the shot, facing the CARDIAC."""
    g, b = places(max(0, i - O0))
    if i < O0:                                            # most of the way in before the orbit, the rest during it
        u = 0.8 * hesitant((i - (O0 - AMIGA_IN)) / AMIGA_IN, HESITATE)
    else:
        u = 0.8 + 0.2 * hesitant((i - O0) / AMIGA_CLOSER, HESITATE_CLOSER)
    if u < 1:
        b = b + aha_camera(0)[1][0] * 700 * (1 - u)        # from off the right of the picture
    base = trans(*b) @ rot_y(facing(b, g)) @ rot_x(math.radians(-12))   # facing the CARDIAC
    BEIGE, KEY, DARK = (228, 222, 202), (204, 198, 180), (70, 70, 74)
    g = 1.0                                               # floating, tipped toward us
    k = base @ np.diag([g, g, g, 1])
    s.box(k @ trans(0, 7, 0), (124, 14, 84), BEIGE, 228, 1)
    s.box(k @ trans(0, 17, 26), (124, 8, 30), BEIGE, 228, 1)
    s.box(k @ trans(0, 14.5, -10), (110, 1.5, 44), DARK, 70, 1)
    for row in range(5):
        for col in range(13):
            s.box(k @ trans(-48 + col * 8, 16, -28 + row * 8.5), (6.6, 2.4, 6.6), KEY, 215, 1)
    s.box(k @ trans(62.4, 9, 10), (0.8, 2.5, 22), (40, 40, 44), 60, 1)
    # a 1084S behind the keyboard, its screen showing the Boing demo's room
    mon = k @ trans(0, 63, 82)
    s.box(mon, (100, 82, 88), BEIGE, 228, 1)
    s.box(mon @ trans(0, -2, 52), (74, 62, 22), BEIGE, 228, 1)                    # the tube's back
    s.box(mon @ trans(0, 3, -44.4), (86, 68, 1), (60, 60, 64), 120, 1)           # the bezel
    s.quad(mon @ trans(0, 3, -45.2), 76, 57, "boing", tag=1)
    s.box(mon @ trans(30, -33, -44.6), (20, 5, 1), (60, 60, 64), 120, 1)         # the badge strip
    if i >= BALL_DROP:                                    # the Boing ball drops in from above and bounces to rest
        t = (i - BALL_DROP) / 12.0
        ball(s, k @ trans(0, 40 + drop_height(t), -12) @ rot_z(math.radians(-18)) @ rot_y(i / 12.0 * 2.4), 24)

BALL_DROP = 228                                         # 0:19 into the sequence: the scene breathes first

def drop_height(t, top=420.0, fall=0.55 / 0.75, keep=0.5):   # at three quarters of its first speed
    """Height above where it comes to rest, t seconds after it's let go from `top`: it falls in `fall` seconds, and
    each bounce keeps `keep` of its speed."""
    g = 2 * top / fall ** 2
    v = g * fall                                          # its speed when it first lands
    if t < fall:
        return top - 0.5 * g * t * t
    t -= fall
    while v > 40:
        v *= keep
        flight = 2 * v / g
        if t < flight:
            return v * t - 0.5 * g * t * t
        t -= flight
    return 0.0

def ball(s, m, r):
    for la in range(8):
        a0, a1 = math.pi * la / 8 - math.pi / 2, math.pi * (la + 1) / 8 - math.pi / 2
        for lo in range(16):
            b0, b1 = 2 * math.pi * lo / 16, 2 * math.pi * (lo + 1) / 16
            p = [(math.cos(a) * math.cos(b) * r, math.sin(a) * r, math.cos(a) * math.sin(b) * r)
                 for a, b in ((a0, b0), (a1, b0), (a1, b1), (a0, b1))]
            pts = np.array(p) @ m[:3, :3].T + m[:3, 3]
            red = (la + lo) % 2 == 0
            centre = pts.mean(0)
            n = centre - m[:3, 3]; n /= np.linalg.norm(n)
            ex = {"tag": 5, "tone": 70 if red else 236}
            col = (226, 32, 32) if red else (244, 244, 244)
            s.tris.append((pts[[0, 1, 2]], col, n, None, ex))
            s.tris.append((pts[[0, 2, 3]], col, n, None, ex))

# ---- the rasteriser: colour, pencil tone, object id, normal, depth, tag -------------------------------------------

NEAR = 2.0

def clip_near(c, uv):
    """A triangle in camera space cut to z >= NEAR (Sutherland-Hodgman against one plane), its texture coordinates
    carried along: the polygon's corners as (point, uv)."""
    out = []
    for k in range(3):
        p, q = c[k], c[(k + 1) % 3]
        up = None if uv is None else uv[k]
        uq = None if uv is None else uv[(k + 1) % 3]
        pin, qin = p[2] >= NEAR, q[2] >= NEAR
        if pin:
            out.append((p, up))
        if pin != qin:
            t = (NEAR - p[2]) / (q[2] - p[2])
            out.append((p + (q - p) * t, None if uv is None else up + (uq - up) * t))
    return out

def raster(tris, eye, basis, hfov, W, H, texs, keep=None):
    fpx = (W / 2) / math.tan(hfov / 2)
    img = np.zeros((H, W, 3), np.float32)
    tone = np.full((H, W), 228.0, np.float32)
    oid = np.zeros((H, W), np.int32)
    nrm = np.zeros((H, W, 3), np.float32)
    izb = np.zeros((H, W), np.float32)
    tag = np.zeros((H, W), np.int8)
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32) + 0.5
    jobs = []
    for n, (pts, rgb, normal, uv, ex) in enumerate(tris, 1):
        if keep is not None and not keep(ex):
            continue
        c = (pts - eye) @ basis.T
        if (c[:, 2] >= NEAR).all():
            jobs.append((n, c, rgb, normal, uv, ex))
        elif (c[:, 2] >= NEAR).any():                     # cut at the near plane, then fan the polygon left
            poly = clip_near(c, uv)
            for k in range(1, len(poly) - 1):
                cc = np.array([poly[0][0], poly[k][0], poly[k + 1][0]])
                uu = None if uv is None else np.array([poly[0][1], poly[k][1], poly[k + 1][1]])
                jobs.append((n, cc, rgb, normal, uu, ex))
    for n, c, rgb, normal, uv, ex in jobs:
        sx = W / 2 + fpx * c[:, 0] / c[:, 2]
        sy = H / 2 - fpx * c[:, 1] / c[:, 2]
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
        zb = izb[y0:y1, x0:x1]
        win = inside & (iz > zb)
        if not win.any():
            continue
        if uv is not None:
            u = (l0 * uv[0, 0] / c[0, 2] + l1 * uv[1, 0] / c[1, 2] + l2 * uv[2, 0] / c[2, 2]) / iz
            v = (l0 * uv[0, 1] / c[0, 2] + l1 * uv[1, 1] / c[1, 2] + l2 * uv[2, 1] / c[2, 2]) / iz
            colour, ptone, cover = texs[ex["tex"]]
            th, tw = colour.shape[:2]
            tu = np.clip((u * tw).astype(int), 0, tw - 1)
            tv = np.clip((v * th).astype(int), 0, th - 1)
            if ex.get("cutout"):
                win &= cover[tv, tu]
                if not win.any():
                    continue
            img[y0:y1, x0:x1][win] = colour[tv, tu][win]
            tone[y0:y1, x0:x1][win] = ptone[tv, tu][win]
        else:
            k = 0.62 + 0.38 * max(0.0, float(normal @ mtv.LIGHT)) + 0.08 * max(0.0, float(-normal[2]))
            img[y0:y1, x0:x1][win] = np.minimum(255, np.array(rgb, np.float32) * k)
            t = ex.get("tone")
            tone[y0:y1, x0:x1][win] = (np.mean(rgb) if t is None else t) * (0.8 + 0.2 * k)
        zb[win] = iz[win]
        oid[y0:y1, x0:x1][win] = n if uv is None else n
        nrm[y0:y1, x0:x1][win] = normal
        tag[y0:y1, x0:x1][win] = ex.get("tag", 0)
    return img, tone, oid, nrm, izb, tag

# ---- the pencil: the video's random-sketch rotoscope ----------------------------------------------------------------

def vnoise(rng, H, W, cell):
    """Smooth noise: a random grid every `cell` pixels, blended between its points."""
    g = rng.random((H // cell + 3, W // cell + 3))
    fy, fx = np.arange(H) / cell, np.arange(W) / cell
    y0, x0 = fy.astype(int), fx.astype(int)
    ty, tx = (fy - y0)[:, None], (fx - x0)[None, :]
    a, b = g[y0][:, x0], g[y0][:, x0 + 1]
    c, d = g[y0 + 1][:, x0], g[y0 + 1][:, x0 + 1]
    return (a * (1 - tx) + b * tx) * (1 - ty) + (c * (1 - tx) + d * tx) * ty

def box_blur(a, size):
    """The mean over a size x size square round each pixel."""
    r = size // 2
    for ax in (0, 1):
        pad = [(r + 1, r) if k == ax else (0, 0) for k in (0, 1)]
        c = np.cumsum(np.pad(a, pad, mode="edge"), axis=ax)
        n = a.shape[ax]
        a = (np.take(c, np.arange(size, size + n), axis=ax) - np.take(c, np.arange(0, n), axis=ax)) / size
    return a

def pencil(tone, oid, nrm, izb, tag, i, K):
    """Tapered, broken contour strokes that boil every frame; dark areas scribbled in with dense hatch strokes at
    each object's own angle; mid tones with sparse strokes; loose stray marks on the walls and floor; the frame's
    edge and the darkest parts solid ink."""
    H, W = tone.shape
    rng = np.random.default_rng(1000 + i)
    ys, xs = np.mgrid[0:H, 0:W]
    depth = np.where(izb > 0, 1 / np.maximum(izb, 1e-9), 1e6)
    e = np.zeros((H, W), bool)
    for dy, dx in ((0, 1), (1, 0)):
        o2 = np.roll(oid, (-dy, -dx), (0, 1))
        e |= o2 != oid
        e |= (np.abs(np.roll(nrm, (-dy, -dx), (0, 1)) - nrm).sum(2) > 0.6) & (tag != 5)   # not between the ball's facets
        e |= np.abs(np.roll(tone, (-dy, -dx), (0, 1)) - tone) > 45
        e |= np.abs(np.roll(depth, (-dy, -dx), (0, 1)) - depth) > 0.05 * depth
    # boil: the whole drawing is redrawn each frame a pixel or two off, differently in different places
    dx = np.rint((vnoise(rng, H, W, 12 * K) - 0.5) * 0.9 * K).astype(int)
    dy = np.rint((vnoise(rng, H, W, 12 * K) - 0.5) * 0.9 * K).astype(int)
    e = e[np.clip(ys + dy, 0, H - 1), np.clip(xs + dx, 0, W - 1)]
    t = tone[np.clip(ys + dy, 0, H - 1), np.clip(xs + dx, 0, W - 1)]
    o = oid[np.clip(ys + dy, 0, H - 1), np.clip(xs + dx, 0, W - 1)]
    # weight: thicker in places (the pressure of the pencil), and broken in places
    thick = vnoise(rng, H, W, 6 * K) > (0.6 if K > 2 else 0.78)
    grow = e | np.roll(e, 1, 0) | np.roll(e, 1, 1)
    line = np.where(thick, grow, e) & (vnoise(rng, H, W, 10 * K) > 0.16)   # long strokes, an occasional gap
    # scribbled darks: each object hatched at its own angle, which changes now and then
    angs = np.radians(np.array([38, 52, 64, 118, 128, 142]))
    pick = (o * 7919 + (i // 3) * 104729) % 6
    ca, sa = np.cos(angs[pick]), np.sin(angs[pick])
    proj = xs * ca + ys * sa
    jag = (vnoise(rng, H, W, 2 * K) - 0.5) * 40
    dark = (t + jag) < 100
    solid = t < 45
    spacing = 0.9 * K + 1.2
    dense = (((proj + rng.uniform(0, 10)) % spacing) < spacing * 0.55) & (vnoise(rng, H, W, 2 * K) > 0.18)
    mid = ((t + jag) < 165) & ~dark
    sparse = (((proj + rng.uniform(0, 10)) % (3.2 * K)) < 0.45 * K + 0.6) & (vnoise(rng, H, W, 5 * K) > 0.62)
    amount = np.zeros((H, W), np.float32)
    amount = np.maximum(amount, (dark & dense) * 0.85)
    amount = np.maximum(amount, (mid & sparse) * 0.6)
    amount = np.maximum(amount, solid * 0.95)
    amount = np.maximum(amount, line * 0.88)
    # stray marks on the walls and floor: loose strokes and loops, a new set every other frame
    room = tag == 2
    srng = np.random.default_rng(5000 + i // 2)
    marks = np.zeros((H, W), bool)
    for _ in range(int(0.00006 * H * W) + 6):
        x, y = srng.uniform(0, W), srng.uniform(0, H)
        if not room[int(y), int(x)]:
            continue
        a, bend, n = srng.uniform(0, 2 * math.pi), srng.uniform(-0.06, 0.06), int(srng.uniform(8, 30) * K)
        loopy = srng.random() < 0.25
        for _ in range(n):
            xi, yi = int(x), int(y)
            if 0 <= xi < W and 0 <= yi < H:
                marks[yi, xi] = True
            x += math.cos(a); y += math.sin(a)
            a += bend * (6 if loopy else 1)
    marks &= room
    if K > 2:
        marks |= np.roll(marks, 1, 0) & room
    amount = np.maximum(amount, marks * 0.7)
    return PAPER + (INK - PAPER) * amount[..., None]

# ---- the camera and the frame's pose --------------------------------------------------------------------------------

TV_WALL = (np.array([150.0, 230.0, 540.0]), 24.0, -12.0)   # on the shop wall: position, yaw, tip
TV_FREE = (CENTRE, 0.0, 0.0)                               # standing free, square to the room
VFOV_SHOP = 2 * math.atan(math.tan(math.radians(38)) / 2)  # mtv.py's 76 degrees across a 2:1 picture
VFOV_AHA = 2 * math.atan(360.0 / AHA["f"])                 # the video's lens, measured

def tv_at(t):
    p = lerp(TV_WALL[0], TV_FREE[0], t)
    yaw, tip = (lerp(TV_WALL[k], TV_FREE[k], t) for k in (1, 2))
    return trans(*p) @ rot_y(math.radians(yaw)) @ rot_x(math.radians(tip))

def tv_size(t):
    """The opening: the shop's 4:3 set squaring up as it comes off the wall."""
    return lerp(TVW, S, t), lerp(TVH, S, t)

def catmull(keys, vals, t):
    """Catmull-Rom through (keys, vals) at t, clamped at the ends."""
    t = min(max(t, keys[0]), keys[-1])
    j = min(len(keys) - 2, max(0, int(np.searchsorted(keys, t, side="right")) - 1))
    p0, p1, p2, p3 = (vals[max(0, j - 1)], vals[j], vals[j + 1], vals[min(len(vals) - 1, j + 2)])
    u = (t - keys[j]) / (keys[j + 1] - keys[j])
    return 0.5 * ((2 * p1) + (-p0 + p2) * u + (2 * p0 - 5 * p1 + 4 * p2 - p3) * u * u + (-p0 + 3 * p1 - 3 * p2 + p3) * u ** 3)

def aha_pose(t):
    """The video's camera at orbit frame t (0..84): a, d, h, yaw, pitch, roll, from the measured keyframes."""
    keys = np.array(AHA["keys"], float)
    names = ("a", "d", "h", "yaw", "pitch", "roll")
    return [catmull(keys, np.array([c[n] for c in AHA["cams"]]), t) for n in names]

def aha_camera(t):
    a, d, h, yaw, pitch, roll = aha_pose(t)
    eye = CENTRE + S * np.array([d * math.sin(a), h, -d * math.cos(a)])
    return eye, rotation(yaw, pitch, roll), a

def look(eye, tgt):
    f = tgt - eye; f /= np.linalg.norm(f)
    r = np.cross([0, 1.0, 0], f); r /= np.linalg.norm(r)
    return np.array([r, np.cross(f, r), f])

def shot(i):
    """camera eye, basis (right, up, forward), vertical field of view, picture aspect, how far the TV is off the wall,
    and the camera's angle round the frame (None until the portal)"""
    shop_eye = np.array([0.0, 115.0, 0.0])
    shop_basis = look(shop_eye, shop_eye + np.array([0, math.sin(math.radians(8)), math.cos(math.radians(8))]))
    if i < SHOP + FADE:
        return shop_eye, shop_basis, VFOV_SHOP, 2.0, 0.0, None
    eye0, basis0, a0 = aha_camera(0)
    if i < SHOP + FADE + MOVE:
        e = ease((i - SHOP - FADE) / (MOVE - 1))
        eye = lerp(shop_eye, eye0, e)
        basis = look(eye, eye + lerp(shop_basis[2], basis0[2], e))
        return eye, basis, lerp(VFOV_SHOP, VFOV_AHA, e), 2.0, e, a0
    eye, basis, a = aha_camera(max(0, i - O0))
    return eye, basis, VFOV_AHA, 2.0, 1.0, a        # the film's 2:1 throughout; the video's vertical framing

# ---- a frame ---------------------------------------------------------------------------------------------------------

STREAMERS_UNTIL = 42                                     # orbit frame: the cyan streamers go at the midway point

def textures(i, tvpic, neon, drawn_card=False):
    lit = (i // 2) % 2 == 0
    tint = np.array([1.0, 0.28, 0.28]) if lit else np.array([0.3, 0.42, 1.0])
    face = np.clip(FACE * tint * 1.15 + 10, 0, 255)
    ftone = FACE.mean(2) * 1.05
    look, tone, cover, card, bare, bare_cover = neon
    if i - O0 >= STREAMERS_UNTIL:
        look, cover = bare, bare_cover
    return {"face": (face, ftone, None), "tvpic": tvpic, "neon": (look, tone, card if drawn_card else cover), "boing": BOING}

def render(i, W, H, neons):
    """Frame i at W x H (2:1 throughout)."""
    eye, basis, vfov, aspect, off, a = shot(i)
    Wi = min(W, int(round(H * aspect / 2)) * 2)
    img = render_at(i, Wi, H, eye, basis, vfov, off, a, neons)
    out = np.zeros((H, W, 3), np.float32)
    out[:, (W - Wi) // 2:(W - Wi) // 2 + Wi] = img
    return out

def render_at(i, W, H, eye, basis, vfov, off, a, neons):
    hfov = 2 * math.atan(math.tan(vfov / 2) * W / H)
    tvm = tv_at(off)
    girl_side = (np.linalg.inv(tvm) @ np.append(eye, 1.0))[2] > 0     # the camera on her side: she's drawn
    neon = neons[i % len(neons)]
    texs = textures(i, (neon[0], neon[0].mean(2), np.ones(neon[0].shape[:2], bool)), neon, girl_side)
    cam = (eye, basis, hfov, W, H, texs)
    if i < SHOP:
        s = Scene(); shop(s, i, tvm)
        return raster(s.tris, eye, basis, hfov, W, H, texs)[0]
    if i < SHOP + FADE:                                   # the shop fading into the drawing
        s = Scene(); shop(s, i, tvm)
        colour = raster(s.tris, eye, basis, hfov, W, H, texs)[0]
        f = ease((i - SHOP + 1) / FADE)
        return lerp(colour, drawing(i, tvm, off, a, shop_room, cam), f)
    if i < SHOP + FADE + MOVE:                            # the TV to the portal; the shop's lines give way to the studio's
        u = ease((i - SHOP - FADE) / (MOVE - 1))
        return lerp(drawing(i, tvm, off, a, shop_room, cam), drawing(i, tvm, off, a, studio_room, cam), u)
    return drawing(i, tvm, off, a, studio_room, cam)

def drawing(i, tvm, off, a, room, cam):
    """The scene in the video's rule: drawn, except what is seen through the opening, which is in colour, on black."""
    eye, basis, hfov, W, H, texs = cam
    K = max(1, H // 96)
    s = Scene()
    room(s)
    frame(s, tvm, off)
    cardiac(s, i, tvm, off)
    if i >= O0 - AMIGA_IN:
        amiga(s, i)
    img, tone, oid, nrm, izb, tag = raster(s.tris, eye, basis, hfov, W, H, texs)
    img[tag == 2] = 0                                     # in colour, the room is black: only the characters show
    # through the opening? the line of sight crosses the frame's plane inside it before meeting anything
    fpx = (W / 2) / math.tan(hfov / 2)
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32) + 0.5
    d_cam = np.stack([(xs - W / 2) / fpx, -(ys - H / 2) / fpx, np.ones_like(xs)], -1).astype(np.float64)
    inv = np.linalg.inv(tvm)
    e_l = inv[:3, :3] @ eye + inv[:3, 3]
    d_l = np.einsum("hwc,cd,ed->hwe", d_cam, basis, inv[:3, :3])   # camera ray -> world -> the frame's own axes
    t0 = -e_l[2] / np.where(np.abs(d_l[..., 2]) < 1e-9, 1e-9, d_l[..., 2])
    p = e_l + d_l * t0[..., None]
    depth = np.where(izb > 0, 1 / np.maximum(izb, 1e-9), 1e9)
    ow, oh = tv_size(off)
    through = (t0 > 0) & (t0 < depth - 0.5) & (np.abs(p[..., 0]) < ow / 2) & (np.abs(p[..., 1]) < oh / 2)
    drawn = pencil(tone, oid, nrm, izb, tag, i, K)
    # the light through the frame: strong while we're in front
    front = 1.0 if a is None else min(1.0, max(0.25, 0.5 + 0.5 * math.cos(a) * 1.6))   # a: the camera's angle round the frame
    glow = box_blur(through.astype(np.float32), int(9 * K) * 2 + 1)
    drawn = drawn + (glow * (1 - through) * front * 0.55)[..., None] * (np.array([255, 250, 255.0]) - drawn)
    return np.where(through[..., None], img, drawn)

def main():
    out = sys.argv[1]
    W, H = map(int, (sys.argv[2] if len(sys.argv) > 2 else "768x384").split("x"))
    first, count = (map(int, sys.argv[3].split(":")) if len(sys.argv) > 3 else (0, FRAMES))
    layer = np.fromfile(mtv.CARDLAYER, np.uint8).reshape(-1, 96, 192, 3)
    rng = np.random.default_rng(1985)
    neons = []
    for k in range(len(layer)):
        c = layer[k].astype(int)
        card = ~((c[..., 1] > 200) & (c[..., 0] < 60) & (c[..., 2] < 60))
        look, cover = mtv.neon(c, card, rng)
        bare, bare_cover = mtv.neon(c, card, rng, strokes=False)   # without the cyan streamers
        # drawn, it's the card itself: pale where the card is pale, inked where it's inked; the glow and strokes are
        # colour only, so in pencil they're paper
        tone = np.where(card, np.minimum(255, c.max(2) * 1.15), 235).astype(np.float32)
        tone = box_blur(tone, 5)                          # simplified, as a hand would draw it: three tones, no texture
        tone = np.where(tone > 175, 235, np.where(tone > 110, 150, 75))
        neons.append((look, tone, cover, card, bare, bare_cover))   # drawn, the card alone
    with open(out, "wb") as fh:
        for i in range(first, first + count):
            fh.write(np.clip(render(i, W, H, neons), 0, 255).astype(np.uint8).tobytes())

if __name__ == "__main__":
    main()
