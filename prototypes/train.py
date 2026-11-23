"""Design prototype: the bullet train at 1536x768. The cab of a Shinkansen at about 300 km/h on a
two-line track, in the mock's order and timing (shinkansen.py): the city by day (lit and shaded buildings, their
glass reflecting the sky, shadows), the tunnel (lamps streaming past, their glow and streaks, the exit blazing),
out at sunset over the paddies (their water mirroring the sky and the sun), the S-curve with the cab leaning, an
oncoming train passing (a long N700-style nose), the truss bridge over a wide river, land again into the sun.
Over the city: the brag, REAL-TIME RAY TRACING / 1536 x 768, and a GLSL ray-generation shader (GL_EXT_ray_tracing).

How it's made, with CARDIAC in mind (floating point here):
  - the ground is analytic per pixel: one plane per row band (deck, paddies, river), the track, rails, paddies,
    dikes and roads are functions of where the ray lands (box-filtered stripes, no texture memory);
  - the sky is a function of direction only (gradient, sun, a cloud layer, the far mountains as a silhouette by
    azimuth), so water and glass reflect it by evaluating it again for the mirrored direction;
  - everything standing up (buildings, masts, the tunnel, the truss, houses, trees, the other train) is triangles
    rasterised into a depth buffer, then shaded per pixel from position and normal (deferred);
  - wires are anti-aliased lines; motion blur gathers along each pixel's screen velocity (which the camera's
    motion gives analytically); the glow is a blurred copy of what's too bright.
Metres; y up; the line runs along +z; our line at x = 0, the other on the right at x = X2.

    python train.py OUT.mp4 [--size 1536x768] [--jobs 12] [--frames A:B]
    python train.py --stills K,K,K OUTDIR [--size 768x384]
"""
import math
import os
import subprocess
import sys

import numpy as np


def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


np.seterr(all='ignore')

FPS, SPEED = 12, 7.0                                      # 7 m a frame: 84 m/s, about 300 km/h
TUN0, TUN1 = 260.0, 500.0                                 # the tunnel
FARM = TUN1
BR0, BLEN = 920.0, 120.0                                  # the bridge, 120 m as in the mock
BR1 = BR0 + BLEN
RIV0, RIV1 = BR0 + 8.0, BR1 - 8.0                         # the water; its banks slope down 10 m outside
BANK = 10.0
END = BR1 + 230.0
X2, MID = 4.3, 2.15                                       # the other line, on the right; between the two
MEET = FARM + 50.0                                        # where we pass the train coming the other way
CARS, CAR = 16, 25.0                                      # a full N700 set, 400 m
BEND0, BEND1, SHIFT = FARM + 30.0, BR0 - 20.0, 70.0     # the S: the line moves 70 m over
EYE_Y = 3.0
HFOV = math.radians(78)
LAND, RIVER, STREET = -1.5, -4.5, -6.5
NFRAMES = int((END - 60) / SPEED)
NEAR = 0.3
SHUTTER = 0.4                                             # of a frame, for the motion blur

# ---- the line: the S-curve, the camera ------------------------------------------------------------------------------

def bend(z):
    u = np.clip((np.asarray(z, float) - BEND0) / (BEND1 - BEND0), 0, 1)
    return SHIFT * (1 - np.cos(np.pi * u)) / 2

def heading(z, dz=2.0):
    return (bend(z + dz) - bend(z - dz)) / (2 * dz)

def look(eye, tgt):
    f = tgt - eye; f = f / np.linalg.norm(f)
    r = np.cross([0, 1.0, 0], f); r /= np.linalg.norm(r)
    return np.array([r, np.cross(f, r), f])

class Cam:
    def __init__(self, t, W, H):
        # the cab's place along the line (the departure's hook: from standstill in a station), how much it sways
        z, sk = HOOKS["cab"](t) if HOOKS.get("cab") else (t * SPEED, 1.0)
        self.t, self.z, self.W, self.H = t, z, W, H
        sway = (0.035 * math.sin(t * 0.7) + 0.012 * math.sin(t * 2.3)) * sk
        self.eye = np.array([float(bend(z)) + sway, EYE_Y + 0.012 * math.sin(t * 1.9), z])
        fwd = np.array([float(heading(z + 25)), -0.05, 1.0])
        b = look(self.eye, self.eye + fwd)
        roll = -0.9 * float(heading(z + 10) - heading(z - 10)) + 0.002 * math.sin(t * 1.3)
        c, s = math.cos(roll), math.sin(roll)
        r, u = b[0].copy(), b[1].copy()
        b[0], b[1] = c * r + s * u, -s * r + c * u
        self.basis = b
        self.f = (W / 2) / math.tan(HFOV / 2)
        dbg = os.environ.get("CAMDEBUG")                  # x,y,z,tx,ty,tz: look from somewhere else
        if dbg:
            v = [float(q) for q in dbg.split(",")]
            self.eye = np.array(v[:3]); self.basis = look(self.eye, np.array(v[3:6]))

    def project(self, P):
        c = (P - self.eye) @ self.basis.T
        z = np.maximum(c[..., 2], 1e-3)
        return self.W / 2 + self.f * c[..., 0] / z, self.H / 2 - self.f * c[..., 1] / z, c[..., 2]

def train_front(t):
    """Where the other train's nose is at frame t: at MEET when we are, closing at our speed."""
    return MEET + (MEET / SPEED - t) * SPEED

# ---- light: day before the tunnel, sunset after -----------------------------------------------------------------------

def nrm(v):
    v = np.asarray(v, float); return v / np.linalg.norm(v)

class Env:
    pass

DAY = Env()
DAY.sun = nrm([-0.30, 0.86, -0.40])
DAY.sun_col = np.array([2.3, 2.2, 2.0])
DAY.amb = np.array([0.24, 0.30, 0.42])
DAY.bounce = np.array([0.20, 0.18, 0.15])
DAY.haze = np.array([0.70, 0.80, 0.95])
DAY.fog = 1400.0
DUSK = Env()
DUSK.sun = nrm([-0.33, 0.10, 1.0])
DUSK.sun_col = np.array([2.6, 1.05, 0.35])
DUSK.amb = np.array([0.14, 0.11, 0.17])
DUSK.bounce = np.array([0.08, 0.05, 0.04])
DUSK.haze = np.array([1.1, 0.48, 0.22])
DUSK.fog = 3200.0

def env_at(z):
    return DAY if z < TUN0 + 10 else DUSK

def exposure(z):
    def ramp(a, b, x):
        return float(np.clip((x - a) / (b - a), 0, 1))
    e = 0.78 + 1.62 * ramp(TUN0 - 5, TUN0 + 45, z)           # eyes opening up in the dark
    e = e - (2.4 - 0.9) * ramp(TUN1 - 5, TUN1 + 60, z) ** 0.7
    return e

# ---- noise --------------------------------------------------------------------------------------------------------

def hash2(a, b):
    return (np.sin(a * 127.1 + b * 311.7) * 43758.5453) % 1.0

def vnoise(x, y):
    xi, yi = np.floor(x), np.floor(y)
    fx, fy = x - xi, y - yi
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
    a, b = hash2(xi, yi), hash2(xi + 1, yi)
    c, d = hash2(xi, yi + 1), hash2(xi + 1, yi + 1)
    return (a + (b - a) * fx) * (1 - fy) + (c + (d - c) * fx) * fy

def fbm(x, y, octaves=4, lac=2.07, gain=0.5):
    s, a, tot = 0.0, 1.0, 0.0
    for o in range(octaves):
        s = s + a * vnoise(x + o * 17.3, y - o * 9.1)
        tot += a; a *= gain; x = x * lac + 0.31 * y; y = y * lac - 0.31 * x / lac
    return s / tot

def smooth(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)

def fstripe(u, period, duty, fw):
    """The share of the pixel's footprint [u - fw/2, u + fw/2] where frac(u / period) < duty: a box-filtered stripe."""
    fw = np.maximum(fw, 1e-4)
    a, b = (u - fw / 2) / period, (u + fw / 2) / period
    F = lambda x: np.floor(x) * duty + np.minimum(x - np.floor(x), duty)
    return np.clip((F(b) - F(a)) / (b - a), 0, 1)

def fband(u, c, hw, fw):
    """The share of the footprint inside [c - hw, c + hw]."""
    fw = np.maximum(fw, 1e-4)
    lo, hi = np.maximum(u - fw / 2, c - hw), np.minimum(u + fw / 2, c + hw)
    return np.clip((hi - lo) / fw, 0, 1)

# ---- the sky: a function of direction only ------------------------------------------------------------------------

def sky_day(d, eye):
    up = np.clip(d[..., 1], 0, 1)
    t = np.sqrt(up)[..., None]
    col = np.array([0.78, 0.86, 0.98]) * 1.15 * (1 - t) + np.array([0.20, 0.40, 0.88]) * t
    near = np.clip(d @ DAY.sun, 0, 1)
    col = col + (near ** 300 * 20 + near ** 12 * 0.35)[..., None] * np.array([1.0, 0.92, 0.75])
    k = np.where(d[..., 1] > 0.015, 1300.0 / np.maximum(d[..., 1], 0.015), 0)
    cx, cz = (eye[0] + d[..., 0] * k) / 900, (eye[2] + d[..., 2] * k) / 900
    n = fbm(cx, cz, 5)
    s = DAY.sun
    n2 = fbm(cx + s[0] * 0.15, cz + s[2] * 0.15, 5)
    cover = smooth(0.52, 0.72, n) * smooth(0.015, 0.09, d[..., 1])
    lit = np.clip(0.75 - (n2 - n) * 5.0, 0, 1)[..., None]
    cc = np.array([0.50, 0.56, 0.68]) * (1 - lit) + np.array([1.55, 1.55, 1.5]) * lit
    far = smooth(0.0, 0.25, d[..., 1])[..., None]                 # low clouds melt into the haze
    cc = cc * far + col * (1 - far)
    return col * (1 - cover[..., None]) + cc * cover[..., None]

FUJI_AZ, FUJI_H, FUJI_W = 0.36, 0.075, 0.30

def ridges(az):
    """The far silhouettes by azimuth: Fuji, a range of hills, a tree line. Heights in elevation (radians)."""
    da = np.abs(az - FUJI_AZ) / FUJI_W
    fuji = FUJI_H * np.clip(1 - da, 0, 1) ** 1.7
    fuji = np.minimum(fuji, FUJI_H * 0.93)
    hills = 0.010 + 0.016 * fbm(az * 6 + 3.0, 0.5, 3) * smooth(-0.2, 0.6, np.abs(az + 0.1))
    trees = 0.0035 + 0.004 * fbm(az * 90, 1.7, 2)
    return fuji, hills, trees

def sky_dusk(d, eye, clouds=True):
    el = d[..., 1]
    up = np.clip(el, -0.02, 1)
    t = np.clip(up / 0.45, 0, 1)[..., None]
    sh_ = DUSK.sun * np.array([1, 0, 1]); sh_ = sh_ / np.linalg.norm(sh_)
    hz = d * np.array([1, 0, 1]); hz = hz / np.maximum(np.linalg.norm(hz, axis=-1, keepdims=True), 1e-6)
    w = ((hz @ sh_ + 1) / 2)[..., None] ** 1.5             # toward the sun warm and bright, away from it cool pink
    low = np.array([0.42, 0.26, 0.42]) * (1 - w) + np.array([1.35, 0.50, 0.16]) * w
    mid = np.array([0.30, 0.14, 0.30]) * (1 - w) + np.array([0.55, 0.16, 0.30]) * w
    high = np.array([0.05, 0.05, 0.22])
    col = np.where(t < 0.3, low + (mid - low) * (t / 0.3), mid + (high - mid) * np.clip((t - 0.3) / 0.7, 0, 1) ** 0.8)
    near = np.clip(d @ DUSK.sun, 0, 1)
    col = col + (near ** 2000 * 60 + near ** 120 * 2.5 + near ** 8 * 0.45)[..., None] * np.array([1.0, 0.62, 0.28])
    az = np.arctan2(d[..., 0], d[..., 2])
    if clouds:
        k = np.where(el > 0.012, 1800.0 / np.maximum(el, 0.012), 0)
        cx, cz = (eye[0] + d[..., 0] * k) / 1500, (eye[2] + d[..., 2] * k) / 2600
        n = fbm(cx * 1.0 + 0.3 * fbm(cx * 0.5, cz * 0.5, 2), cz * 2.2, 5)
        band = smooth(0.02, 0.10, el) * (1 - 0.6 * smooth(0.45, 0.8, el))
        dens = smooth(0.50, 0.78, n) * band
        thin = 1 - smooth(0.5, 0.85, n)
        fwd = near[..., None]
        base = np.array([0.16, 0.07, 0.15]) * (1 - thin[..., None]) + np.array([1.3, 0.42, 0.42]) * thin[..., None]
        gold = np.array([3.6, 1.6, 0.5]) * (fwd ** 5 * (0.2 + 1.2 * thin[..., None]))
        hl = smooth(0.0, 0.25, el)[..., None]              # higher clouds catch more pink
        cc = base * (0.6 + 0.6 * hl) + gold
        col = col * (1 - dens[..., None]) + cc * dens[..., None]
    fuji, hills, trees = ridges(az)
    haze = col
    e = np.abs(el)                                         # reflections ask with the elevation mirrored back up
    on_f = (e < fuji) & (el > -0.5)
    lit = np.clip(0.5 - (az - FUJI_AZ) * 6, 0, 1)[..., None]  # the sun is to the left: its left flank lit
    rock = np.array([0.20, 0.14, 0.30]) * (1 - lit) + np.array([0.55, 0.28, 0.32]) * lit
    snow = np.array([0.55, 0.42, 0.70]) * (1 - lit) + np.array([1.8, 0.85, 0.75]) * lit
    snowy = (e > fuji - FUJI_H * (0.30 + 0.05 * np.sin(az * 140)))[..., None]
    fc = np.where(snowy, snow, rock)
    fc = fc * 0.55 + haze * 0.45
    col = np.where(on_f[..., None], fc, col)
    hc = np.array([0.22, 0.13, 0.25]) * 0.6 + haze * 0.4
    col = np.where((e < hills)[..., None], hc, col)
    tc = np.array([0.07, 0.05, 0.08]) * 0.75 + haze * 0.25
    col = np.where((e < trees)[..., None], tc, col)
    return col

def sky(d, eye, env):
    return sky_day(d, eye) if env is DAY else sky_dusk(d, eye)

# ---- geometry -----------------------------------------------------------------------------------------------------

# material kinds
K_SKY, K_DECK, K_PADDY, K_WATER, K_BANK, K_STREET, K_EMBANK = range(7)
K_BUILD, K_CONC, K_HILL, K_TUNNEL, K_LAMP, K_METAL, K_TRUSS, K_TRAIN, K_TREE, K_HOUSE, K_TILE, K_SIGN, K_WOOD, \
    K_SHED, K_DARK, K_ROOFTOP, K_CHOUSE, K_SOLAR, K_ASPHALT = range(10, 29)

class Mats:
    def __init__(self):
        self.kind, self.alb, self.p = [], [], []
        for k in range(7):                                # the ground's, one each, ids 0..6
            self.add(k, (0, 0, 0))

    def add(self, kind, alb, p=(0, 0, 0, 0)):
        self.kind.append(kind); self.alb.append(alb); self.p.append(tuple(p) + (0,) * (6 - len(p)))
        return len(self.kind) - 1

    def arrays(self):
        return np.array(self.kind, np.int16), np.array(self.alb, np.float32), np.array(self.p, np.float32)

class Geo:
    def __init__(self):
        self.V, self.N, self.M = [], [], []
        self.chunks = []                                  # instanced models: (V, N, M) arrays

    def place(self, model, pos, rot=0.0, scale=(1.0, 1.0, 1.0), roles=None, lod=0):
        """A model from the library, turned rot radians about y, scaled, set down at pos; roles maps its
        material names to material ids (a name it doesn't map is dropped)."""
        V, N, R, names = model
        sc = np.asarray(scale, float)
        c, s = math.cos(rot), math.sin(rot)
        Ry = np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])
        ids = np.array([roles.get(nm, -1) for nm in names])[R]
        keep = ids >= 0
        V2 = (V[keep] * sc) @ Ry.T + np.asarray(pos, float)
        N2 = (N[keep] / sc) @ Ry.T
        N2 /= np.maximum(np.linalg.norm(N2, axis=-1, keepdims=True), 1e-9)
        self.chunks.append((V2, N2, ids[keep], lod, float(pos[2])))

    def tri(self, a, b, c, n, m, vn=None):
        self.V.append((a, b, c)); self.M.append(m)
        self.N.append(vn if vn is not None else (n, n, n))

    def quad(self, a, b, c, d, n, m):
        self.tri(a, b, c, n, m); self.tri(a, c, d, n, m)

    def box(self, lo, hi, m, bottom=False):
        (x0, y0, z0), (x1, y1, z1) = lo, hi
        P = lambda x, y, z: (x, y, z)
        self.quad(P(x0, y0, z0), P(x0, y1, z0), P(x0, y1, z1), P(x0, y0, z1), (-1, 0, 0), m)
        self.quad(P(x1, y0, z0), P(x1, y0, z1), P(x1, y1, z1), P(x1, y1, z0), (1, 0, 0), m)
        self.quad(P(x0, y0, z0), P(x1, y0, z0), P(x1, y1, z0), P(x0, y1, z0), (0, 0, -1), m)
        self.quad(P(x0, y0, z1), P(x0, y1, z1), P(x1, y1, z1), P(x1, y0, z1), (0, 0, 1), m)
        self.quad(P(x0, y1, z0), P(x1, y1, z0), P(x1, y1, z1), P(x0, y1, z1), (0, 1, 0), m)
        if bottom:
            self.quad(P(x0, y0, z0), P(x0, y0, z1), P(x1, y0, z1), P(x1, y0, z0), (0, -1, 0), m)

    def member(self, a, b, w, h, m):
        """A beam from a to b, w wide and h deep."""
        a, b = np.asarray(a, float), np.asarray(b, float)
        t = nrm(b - a)
        s = np.cross(t, [0, 1.0, 0])
        if np.linalg.norm(s) < 1e-3:
            s = np.array([1.0, 0, 0])
        s = nrm(s); u = np.cross(s, t)
        cs = [(-1, -1), (1, -1), (1, 1), (-1, 1)]
        ring = lambda p: [p + s * (w / 2) * i + u * (h / 2) * j for i, j in cs]
        A, B = ring(a), ring(b)
        sides = [(-u), (s), (u), (-s)]
        for k in range(4):
            k2 = (k + 1) % 4
            n = nrm(A[k] + A[k2] - 2 * a)
            self.quad(A[k], B[k], B[k2], A[k2], n, m)
        self.quad(A[0], A[3], A[2], A[1], -t, m)
        self.quad(B[0], B[1], B[2], B[3], t, m)

    def mesh(self, grid, m, closed_u=False):
        """A surface from a grid of points (rows along, columns around), smooth normals, outward taken as away from
        each row's centre."""
        G = np.asarray(grid, float)
        R, C = G.shape[:2]
        fn = np.zeros_like(G)
        cu = C if closed_u else C - 1
        for i in range(R - 1):
            for j in range(cu):
                j2 = (j + 1) % C
                n = np.cross(G[i + 1, j] - G[i, j], G[i, j2] - G[i, j])
                for (a, b) in ((i, j), (i + 1, j), (i, j2), (i + 1, j2)):
                    fn[a, b] += n
        cen = G.mean(1, keepdims=True)
        out = np.sign(np.sum(fn * (G - cen), -1, keepdims=True))
        out[out == 0] = 1
        sgn = 1.0 if np.mean(out) >= 0 else -1.0
        fn = fn * sgn
        fn /= np.maximum(np.linalg.norm(fn, axis=-1, keepdims=True), 1e-9)
        for i in range(R - 1):
            for j in range(cu):
                j2 = (j + 1) % C
                a, b, c, d = G[i, j], G[i + 1, j], G[i + 1, j2], G[i, j2]
                n = nrm(np.cross(b - a, d - a) * sgn) if np.linalg.norm(np.cross(b - a, d - a)) > 1e-9 else fn[i, j]
                self.tri(a, b, c, n, m, (fn[i, j], fn[i + 1, j], fn[i + 1, j2]))
                self.tri(a, c, d, n, m, (fn[i, j], fn[i + 1, j2], fn[i, j2]))

    def arrays(self):
        V = [np.array(self.V, np.float64).reshape(-1, 3, 3)] + [c[0] for c in self.chunks]
        N = [np.array(self.N, np.float64).reshape(-1, 3, 3)] + [c[1] for c in self.chunks]
        M = [np.array(self.M, np.int32)] + [c[2].astype(np.int32) for c in self.chunks]
        n0 = len(self.M)
        self.lod = np.concatenate([np.zeros(n0, np.int8)] + [np.full(len(c[2]), c[3], np.int8) for c in self.chunks])
        self.iz = np.concatenate([np.full(n0, -1e9)] + [np.full(len(c[2]), c[4]) for c in self.chunks])
        return np.concatenate(V), np.concatenate(N), np.concatenate(M)

MODELS_DIR = os.environ.get("TRAIN_MODELS") or _work("models")
_LIB = {}

def model(name):
    """A model built by models/build_models.py: its triangles, their vertex normals, each one's material
    (an index into its list of material names)."""
    if name not in _LIB:
        vs, ns, tris, nrms, mats, names, cur = [], [], [], [], [], [], 0
        for line in open(os.path.join(MODELS_DIR, name + ".obj")):
            if line.startswith("v "):
                vs.append([float(t) for t in line.split()[1:4]])
            elif line.startswith("vn "):
                ns.append([float(t) for t in line.split()[1:4]])
            elif line.startswith("usemtl "):
                nm = line.split()[1]
                if nm not in names:
                    names.append(nm)
                cur = names.index(nm)
            elif line.startswith("f "):
                cs = [c.split("/") for c in line.split()[1:4]]
                tris.append([int(c[0]) - 1 for c in cs]); nrms.append([int(c[2]) - 1 for c in cs]); mats.append(cur)
        vs, ns = np.array(vs), np.array(ns)
        _LIB[name] = (vs[np.array(tris)], ns[np.array(nrms)], np.array(mats), names)
    return _LIB[name]

def bent(V):
    V = V.copy()
    V[..., 0] += bend(V[..., 2])
    return V

CITY_WIRES = []                                            # the suburb's overhead wires: (points, radius)

def build(mats):
    g = Geo()
    rng = np.random.default_rng(1964)                     # the year the Shinkansen opened
    city_trees = []
    # -- the city: a Japanese suburb packed up to the viaduct. Lots along streets parallel to the line: two-storey
    # houses with tiled or tin roofs, small apartment blocks, shop-houses, the odd mid-rise with its signs; utility
    # poles and their wires along every street; taller blocks further off for the skyline.
    pal = [(0.55, 0.53, 0.50), (0.62, 0.56, 0.47), (0.44, 0.43, 0.42), (0.72, 0.71, 0.68), (0.45, 0.32, 0.26),
           (0.36, 0.40, 0.45), (0.60, 0.60, 0.55), (0.50, 0.47, 0.52), (0.66, 0.52, 0.40), (0.58, 0.40, 0.32),
           (0.70, 0.64, 0.52)]
    siding = [(0.74, 0.71, 0.64), (0.66, 0.62, 0.55), (0.78, 0.77, 0.74), (0.58, 0.55, 0.50), (0.70, 0.62, 0.50),
              (0.62, 0.64, 0.62), (0.55, 0.47, 0.40), (0.80, 0.74, 0.62), (0.50, 0.52, 0.55)]
    tiles = [(0.10, 0.11, 0.14), (0.16, 0.13, 0.12), (0.08, 0.10, 0.16), (0.20, 0.20, 0.21), (0.24, 0.14, 0.10)]
    tins = [(0.15, 0.30, 0.55), (0.45, 0.20, 0.12), (0.20, 0.38, 0.30), (0.55, 0.55, 0.55), (0.60, 0.42, 0.22)]
    solar = mats.add(K_SOLAR, (0.03, 0.05, 0.12))
    rooftop = mats.add(K_ROOFTOP, (0.45, 0.45, 0.44))
    cpole = mats.add(K_CONC, (0.55, 0.55, 0.53), (0.0,))
    acu = mats.add(K_ROOFTOP, (0.70, 0.70, 0.68))
    trim = mats.add(K_CONC, (0.62, 0.61, 0.58), (0.0,))
    railm = mats.add(K_CONC, (0.50, 0.51, 0.52), (0.0,))
    tankm = mats.add(K_METAL, (0.55, 0.56, 0.58))
    def bbox(name):
        V = model(name)[0]
        return V[..., 0].min(), V[..., 0].max(), V[..., 2].min(), V[..., 2].max(), V[..., 1].max()
    def fit(name, x0, x1, z0, z1, base, roles, turn=None, sy=1.0):
        """Set a model down on the lot [x0, x1] x [z0, z1], turned a quarter at a time, stretched to fit."""
        mx0, mx1, mz0, mz1, _ = bbox(name)
        mw, md = mx1 - mx0, mz1 - mz0
        q = int(rng.integers(4)) if turn is None else turn
        rot = q * math.pi / 2
        lw, ld = (x1 - x0, z1 - z0) if q % 2 == 0 else (z1 - z0, x1 - x0)
        sx, sz = float(np.clip(lw / mw, 0.75, 1.3)), float(np.clip(ld / md, 0.75, 1.3))
        g.place(model(name), ((x0 + x1) / 2, base, (z0 + z1) / 2), rot, (sx, sy, sz), roles)
    def city_house(x0, x1, z0, z1, floors):
        name = "house_%d" % int(rng.integers(8))
        top = STREET + bbox(name)[4] * 0.8
        wm = mats.add(K_CHOUSE, siding[int(rng.integers(len(siding)))],
                      (float(rng.uniform(0, 100)), top, STREET, float(rng.uniform(2.6, 3.8)), float(rng.integers(0, 3))))
        roles = {"wall": wm, "roof": mats.add(K_TILE, tiles[int(rng.integers(len(tiles)))]),
                 "tin": mats.add(K_SHED, tins[int(rng.integers(len(tins)))]), "trim": trim, "rail": railm,
                 "unit": acu, "solar": solar}
        fit(name, x0, x1, z0, z1, STREET, roles)
    def block(x0, x1, z0, z1, floors, style, signs=True):
        side = -1 if x1 < MID else 1
        if style == 3:
            name = "apartment_%d" % int(rng.integers(4))
            turn = 1 if side < 0 else 3                   # balconies toward the line
        else:
            k = min(3, max(0, (floors - 4) // 2))
            name = "block_%d" % k
            turn = None
        sy = 1.0 if style == 3 else floors / (5, 7, 9, 12)[k]
        h = STREET + bbox(name)[4] * sy
        m = mats.add(K_BUILD, pal[int(rng.integers(len(pal)))],
                     (style, float(rng.uniform(2.4, 3.4)), float(rng.integers(4)), float(rng.uniform(0, 1)), h, STREET))
        fit(name, x0, x1, z0, z1, STREET, {"wall": m, "trim": trim, "rail": railm, "unit": acu, "tank": tankm}, turn, sy)
        if signs and rng.random() < 0.5:                  # a vertical sign on the corner facing down the line
            sm = mats.add(K_SIGN, tuple(rng.choice([(1.0, 0.15, 0.1), (1.0, 0.8, 0.1), (0.15, 0.45, 1.0),
                                                    (0.95, 0.95, 0.95), (0.1, 0.8, 0.4), (0.9, 0.4, 0.6)])),
                          (float(rng.uniform(0, 100)),))
            fx = x1 if side < 0 else x0
            sy = float(rng.uniform(STREET + 4, max(STREET + 5, h - 8)))
            out = 1.3 * (1 if side < 0 else -1)
            xa, xb = sorted((fx, fx + out))
            g.box((xa, sy, z0 + 0.3), (xb, sy + float(rng.uniform(5, 8)), z0 + 0.65), sm)
        if signs and rng.random() < 0.35:                 # a billboard on the roof, facing down the line
            bm = mats.add(K_SIGN, tuple(rng.choice([(0.9, 0.9, 0.88), (0.9, 0.2, 0.15), (0.1, 0.3, 0.8),
                                                    (0.95, 0.75, 0.1)])), (float(rng.uniform(0, 100)), 1.0))
            bw = min(x1 - x0 - 1, 12.0); bx = (x0 + x1) / 2
            g.box((bx - bw / 2, h + 1.4, z0 + 1), (bx + bw / 2, h + 1.4 + float(rng.uniform(3, 5)), z0 + 1.3), bm)
            for px in (bx - bw / 3, bx + bw / 3):
                g.box((px - 0.12, h, z0 + 1.3), (px + 0.12, h + 1.4, z0 + 1.6), rooftop)
    CITY_WIRES.clear()
    def pole_line(x, za, zb):
        """Utility poles along a street every 30 m or so, their wires sagging between them."""
        zs = list(np.arange(za + float(rng.uniform(0, 12)), zb, float(rng.uniform(27, 34))))
        for z in zs:
            g.place(model("pole_tr" if rng.random() < 0.35 else "pole"), (x, STREET, z), float(rng.uniform(-0.2, 0.2)),
                    roles={"conc": cpole, "steel": tankm, "unit": acu})
        for z0, z1 in zip(zs[:-1], zs[1:]):
            for dx, y, sag in ((-0.8, STREET + 10.3, 0.5), (0.0, STREET + 10.3, 0.5), (0.8, STREET + 10.3, 0.5),
                               (-0.6, STREET + 8.7, 0.6), (0.15, STREET + 6.0, 0.8)):
                zz = np.linspace(z0, z1, 9); u = (zz - z0) / (z1 - z0)
                CITY_WIRES.append((np.stack([np.full_like(zz, x + dx), y - sag * 4 * u * (1 - u), zz], 1),
                                   0.012 if y > 3 else 0.02))
        return zs
    zend = TUN0 - 82.0
    for side in (-1, 1):
        # streets parallel to the line: one under the viaduct's side, then every two rows of lots
        bands, x = [], 13.0
        streets = [9.5]
        while x < 175:
            for k in range(2):
                dep = float(rng.uniform(10, 14))
                bands.append((x, x + dep, len(bands)))
                x += dep + float(rng.uniform(0.6, 1.5))
            streets.append(x + 3.0)
            x += 6.5
        for sx in streets[:4]:
            pole_line(MID + side * sx, -140.0, zend)
        for x0b, x1b, bi in bands:
            near = bi < 2
            z = -140.0 + float(rng.uniform(0, 8))
            while z < zend - 4:
                if rng.random() < 0.06:                   # a cross street
                    z += 7.0; continue
                fr = float(rng.uniform(7, 12))
                r = rng.random()
                if side < 0:
                    xa, xb = MID - x1b, MID - x0b
                else:
                    xa, xb = MID + x0b, MID + x1b
                inset = float(rng.uniform(0, 1.5))
                xa2, xb2 = (xa + inset, xb) if side < 0 else (xa, xb - inset)
                pm_ = [0.50, 0.20, 0.12, 0.10, 0.08] if near else [0.58, 0.20, 0.06, 0.06, 0.10]
                kind_ = rng.choice(5, p=pm_)
                if kind_ == 0:
                    city_house(xa2, xb2, z, z + fr - 0.8, int(rng.choice([2, 2, 2, 3])))
                elif kind_ == 1:
                    ln = fr + float(rng.uniform(6, 14))
                    block(xa2, xb2, z, z + ln - 1.0, int(rng.integers(3, 6)), 3, signs=False)
                    fr = ln
                elif kind_ == 2:
                    block(xa2, xb2, z, z + fr - 0.6, 3, 0)
                elif kind_ == 3:
                    ln = fr + float(rng.uniform(8, 18))
                    block(xa2, xb2, z, z + ln - 1.0, int(rng.integers(6, 12)), int(rng.choice([0, 1, 3])))
                    fr = ln
                else:                                     # a car park or a patch of garden: a tree or two
                    for k in range(int(rng.integers(0, 3))):
                        city_trees.append(((xa + xb) / 2 + float(rng.uniform(-3, 3)), z + fr / 2 +
                                           float(rng.uniform(-2, 2))))
                z += fr
        for k in range(26):                               # the skyline: taller blocks further off
            x0b = float(rng.uniform(180, 420)); dep = float(rng.uniform(18, 35)); ln = float(rng.uniform(20, 40))
            zz = float(rng.uniform(-160, zend + 40))
            xa, xb = (MID - x0b - dep, MID - x0b) if side < 0 else (MID + x0b, MID + x0b + dep)
            block(xa, xb, zz, zz + ln, int(rng.integers(8, 28)), int(rng.choice([0, 1, 2, 2])))
    # the viaduct's parapets
    pm = mats.add(K_CONC, (0.58, 0.57, 0.54), (10.0,))
    for z0 in np.arange(-120, TUN0 - 60, 20.0):
        for xa, xb in ((MID - 5.95, MID - 5.65), (MID + 5.65, MID + 5.95)):
            g.box((xa, -0.6, z0), (xb, 1.15, z0 + 20), pm)
    # -- the hill the tunnel runs through, its portal and the cutting
    hillm = mats.add(K_HILL, (0.045, 0.085, 0.03))
    def hh(x, z):
        dx = abs(x - MID)
        behind = float(smooth(0, 90, z - TUN0))
        top = 13 + 45 * behind * math.exp(-(dx / 300) ** 2) + 0.06 * max(0, dx - 12)
        wav = 7 * math.sin(x * 0.021 + 1.3) * math.cos(z * 0.017) + 4 * math.sin(x * 0.05 + z * 0.03)
        top += wav * float(smooth(10, 40, dx)) + 3 * behind * math.sin(x * 0.09)
        if z < TUN0:
            s = float(smooth(TUN0 - 75, TUN0, z))
            top = STREET + (top - STREET) * s
        return top
    xs_ = [MID + k * 12.0 for k in range(-30, 31)]
    zs_ = [TUN0 - 80 + k * 10.0 for k in range(0, 36)]
    Hh = np.array([[hh(x, z) for x in xs_] for z in zs_])
    for i in range(len(zs_) - 1):
        for j in range(len(xs_) - 1):
            xa, xb, za, zb = xs_[j], xs_[j + 1], zs_[i], zs_[i + 1]
            if abs((xa + xb) / 2 - MID) < 12 and zb <= TUN0:
                continue                                  # the cutting
            a, b = (xa, Hh[i, j], za), (xa, Hh[i + 1, j], zb)
            c, d = (xb, Hh[i + 1, j + 1], zb), (xb, Hh[i, j + 1], za)
            n = nrm(np.cross(np.subtract(b, a), np.subtract(d, a)))
            if n[1] < 0:
                n = -n
            g.quad(a, b, c, d, n, hillm)
    # the cutting's walls
    cm = mats.add(K_CONC, (0.52, 0.51, 0.48), (4.0,))
    for i in range(len(zs_) - 1):
        za, zb = zs_[i], zs_[i + 1]
        if zb > TUN0:
            break
        for xw, n in ((MID - 12, (1, 0, 0)), (MID + 12, (-1, 0, 0))):
            ya, yb = hh(xw, za), hh(xw, zb)
            if max(ya, yb) <= 0:
                continue
            g.quad((xw, -1, za), (xw, -1, zb), (xw, max(yb, 0), zb), (xw, max(ya, 0), za), n, cm)
    # the tunnel: a horseshoe, its lining in 20 m rings; the portal face at its mouth
    R, YC = 5.6, 3.5
    sec = [(MID - R, -0.4), (MID - R, YC)]
    for k in range(1, 18):
        a = math.pi - math.pi * k / 18
        sec.append((MID + R * math.cos(a), YC + R * math.sin(a)))
    sec += [(MID + R, YC), (MID + R, -0.4)]
    tm = mats.add(K_TUNNEL, (0.33, 0.32, 0.30))
    mm_t = mats.add(K_METAL, (0.30, 0.30, 0.32))
    for z0 in np.arange(TUN0, TUN1, 20.0):
        z1 = z0 + 20
        for (xa, ya), (xb, yb) in zip(sec[:-1], sec[1:]):
            mx, my = (xa + xb) / 2, (ya + yb) / 2
            n = nrm([MID - mx, (YC - my) if my > YC else 0.0, 0])
            g.quad((xa, ya, z0), (xb, yb, z0), (xb, yb, z1), (xa, ya, z1), n, tm)
        tr = mats.add(K_CONC, (0.45, 0.44, 0.41), (2.0,)) if z0 == TUN0 else tr
        for xa, xb in ((MID - R, MID - R + 0.7), (MID + R - 0.7, MID + R)):   # cable troughs
            g.box((xa, -0.4, z0), (xb, 0.55, z1), tr)
    for zb in np.arange(TUN0 + 25, TUN1, 50.0):            # the overhead line's brackets in the roof
        for xw in (0.0, X2):
            g.box((xw - 0.08, 6.0, zb - 0.08), (xw + 0.08, 8.9, zb + 0.08), mm_t)
            g.box((xw - 0.6, 6.0, zb - 0.06), (xw + 0.6, 6.12, zb + 0.06), mm_t)
    lm = mats.add(K_LAMP, (1.0, 0.72, 0.38))
    dx = math.sqrt(R * R - 0.7 * 0.7)
    for zl in np.arange(TUN0 + 6, TUN1, 12.0):
        for xl, s in ((MID - dx + 0.12, 1), (MID + dx - 0.12, -1)):
            xa, xb = sorted((xl, xl + 0.22 * s))
            g.box((xa, 4.05, zl), (xb, 4.35, zl + 1.3), lm)
    portal = mats.add(K_CONC, (0.60, 0.59, 0.56), (6.0,))
    PW, PH = 13.0, hh(MID, TUN0) - 0.3
    # the face between the arch and a rectangle: columns of quads from the arch up to the top edge
    for (xa, ya), (xb, yb) in zip(sec[:-1], sec[1:]):
        if ya > YC - 1e-6 and yb > YC - 1e-6:
            g.quad((xa, ya, TUN0), (xa, PH, TUN0), (xb, PH, TUN0), (xb, yb, TUN0), (0, 0, -1), portal)
    for s in (-1, 1):
        xa, xb = sorted((MID + s * R, MID + s * PW))
        g.quad((xa, -0.4, TUN0), (xa, PH, TUN0), (xb, PH, TUN0), (xb, -0.4, TUN0), (0, 0, -1), portal)
    g.box((MID - PW - 0.5, PH, TUN0 - 0.8), (MID + PW + 0.5, PH + 0.9, TUN0 + 0.5), portal)   # the hood's coping
    # -- the catenary: masts and a beam every 50 m, both lines; not in the tunnel or on the bridge
    mm = mats.add(K_METAL, (0.40, 0.41, 0.43))
    mbase = mats.add(K_CONC, (0.50, 0.49, 0.47), (0.0,))
    for z in np.arange(-100, END + 400, 50.0):
        if TUN0 - 8 <= z < TUN1 + 5 or BR0 - 5 <= z < BR1 + 5:
            continue
        g.place(model("mast"), (0.0, 0.0, z), 0.0, roles={"steel": mm, "conc": mbase})
    # -- the farmland: fence posts, the farm road's poles, houses, sheds, trees
    post = mats.add(K_CONC, (0.36, 0.36, 0.35), (1.0,))
    for zf in np.arange(FARM + 4, END + 300, 5.0):
        if BR0 - 2 < zf < BR1 + 2:
            continue
        for x in (MID - 7.6, MID + 7.6):
            g.box((x - 0.06, -0.9, zf - 0.06), (x + 0.06, 0.45, zf + 0.06), post)
    pole = mats.add(K_WOOD, (0.30, 0.22, 0.15))
    POLES.clear()
    for zf in np.arange(FARM + 14, END + 400, 34.0):
        if RIV0 - BANK - 5 < zf < RIV1 + BANK + 5:
            continue
        x = MID - 19.0
        g.member((x, LAND, zf), (x, LAND + 10.5, zf), 0.32, 0.32, pole)
        g.box((x - 1.3, LAND + 9.6, zf - 0.08), (x + 1.3, LAND + 9.78, zf + 0.08), pole)
        POLES.append(zf)
    house_wall = [mats.add(K_HOUSE, a, (float(i),)) for i, a in enumerate(
        [(0.78, 0.76, 0.70), (0.70, 0.66, 0.58), (0.62, 0.50, 0.38), (0.80, 0.80, 0.78)])]
    tile = [mats.add(K_TILE, a) for a in [(0.10, 0.11, 0.14), (0.16, 0.13, 0.12), (0.08, 0.10, 0.16)]]
    shed = [mats.add(K_SHED, a) for a in [(0.15, 0.30, 0.55), (0.45, 0.20, 0.12), (0.55, 0.55, 0.55)]]
    trees = [mats.add(K_TREE, a, (float(i),)) for i, a in enumerate(
        [(0.07, 0.15, 0.05), (0.09, 0.18, 0.06), (0.06, 0.12, 0.06)])]
    def house(x, z, w, d, h, rot):
        name = "house_%d" % int(rng.integers(8))
        roles = {"wall": house_wall[int(rng.integers(len(house_wall)))], "roof": tile[int(rng.integers(len(tile)))],
                 "tin": shed[int(rng.integers(len(shed)))], "trim": trim, "rail": railm, "unit": acu, "solar": solar}
        sc = float(np.clip(w / 9.0, 0.85, 1.4))
        g.place(model(name), (x, LAND - 0.1, z), float(rng.integers(4)) * math.pi / 2 + float(rng.uniform(-0.05, 0.05)),
                (sc, float(np.clip(h / 6.0, 0.8, 1.1)), sc), roles)
    trunk = mats.add(K_WOOD, (0.12, 0.08, 0.05))
    # foliage: kind K_TREE, p = (species, seed); species 0 broadleaf, 1 cedar, 2 pine, 3 bamboo
    leaf = {0: [(0.07, 0.15, 0.05), (0.09, 0.17, 0.05), (0.10, 0.15, 0.07)], 1: [(0.035, 0.07, 0.045)],
            2: [(0.05, 0.10, 0.05)], 3: [(0.16, 0.24, 0.07)]}
    def foliage(sp):
        a_ = leaf[sp][int(rng.integers(len(leaf[sp])))]
        return mats.add(K_TREE, a_, (float(sp), float(rng.uniform(0, 100))))
    far_of = {0: "tree_broad_far", 1: "tree_cedar_far", 2: "tree_pine_far", 3: "tree_bamboo_far"}
    def plant(name, sp, x, z, height, base):
        h0 = model(name)[0][..., 1].max()
        sc = height / h0
        rot = float(rng.uniform(0, 6.28))
        roles = {"leaf": foliage(sp), "trunk": trunk}
        g.place(model(name), (x, base, z), rot, (sc, sc, sc), roles, lod=1)
        hf = model(far_of[sp])[0][..., 1].max()
        g.place(model(far_of[sp]), (x, base, z), rot, (height / hf,) * 3, roles, lod=2)
    def broadleaf(x, z, r, h, base=LAND):
        plant("tree_broad_%d" % int(rng.integers(5)), 0, x, z, h * 1.1, base)
    def cedar(x, z, h, base=LAND):
        plant("tree_cedar_%d" % int(rng.integers(3)), 1, x, z, h, base)
    def pine(x, z, h, base=LAND):
        plant("tree_pine_%d" % int(rng.integers(3)), 2, x, z, h, base)
    def bamboo(x, z, n, h, base=LAND):
        plant("tree_bamboo_%d" % int(rng.integers(2)), 3, x, z, h, base)
    def tree(x, z, r, h, base=LAND):
        sp = rng.random()
        if sp < 0.6:
            broadleaf(x, z, r, h, base)
        elif sp < 0.8:
            pine(x, z, h * 0.8, base)
        else:
            cedar(x, z, h * 1.4, base)
    def clear_of_line(x, z, margin):
        return abs(x - MID) > margin and not (RIV0 - BANK - 10 < z < RIV1 + BANK + 10)
    for (cx_, cz_) in city_trees:                         # the suburb's gardens
        broadleaf(cx_, cz_, float(rng.uniform(2, 3.5)), float(rng.uniform(5, 8)), STREET)
    zf = FARM + 25
    while zf < END + 350:                                 # near the line: farmsteads, sheds, groves
        side = 1 if rng.random() < 0.5 else -1
        x = MID + side * float(rng.uniform(16, 45))
        if not clear_of_line(x, zf, 14) or abs(x - (MID - 19)) < 4:
            zf += 10; continue
        r = rng.random()
        if r < 0.45:
            house(x, zf, float(rng.uniform(8, 13)), float(rng.uniform(6, 9)), float(rng.uniform(3.2, 5.5)),
                  bool(rng.random() < 0.5))
            far = x + side * 9
            if rng.random() < 0.5:                        # a windbreak of cedars behind the farmstead
                for k in range(int(rng.integers(5, 9))):
                    cedar(far + float(rng.uniform(-0.6, 0.6)), zf - 12 + k * 3.4, float(rng.uniform(11, 16)))
            else:
                for t in range(int(rng.integers(1, 3))):
                    tree(x + float(rng.uniform(-9, 9)), zf + float(rng.uniform(6, 12)) * (1 if rng.random() < .5 else -1),
                         float(rng.uniform(2.5, 4.0)), float(rng.uniform(6, 10)))
            if rng.random() < 0.3:
                bamboo(far + side * 6, zf + float(rng.uniform(-8, 8)), int(rng.integers(4, 8)), float(rng.uniform(9, 13)))
        elif r < 0.62:
            sm = shed[int(rng.integers(len(shed)))]
            w, d, h = float(rng.uniform(4, 8)), float(rng.uniform(4, 7)), float(rng.uniform(2.5, 4))
            g.box((x - w / 2, LAND - 0.2, zf - d / 2), (x + w / 2, LAND + h, zf + d / 2), sm)
        else:
            for t in range(int(rng.integers(2, 5))):
                tree(x + float(rng.uniform(-6, 6)), zf + float(rng.uniform(-6, 6)),
                     float(rng.uniform(2.5, 4.5)), float(rng.uniform(6, 11)))
        zf += float(rng.uniform(16, 32))
    # rows of trees along the paddies: along canal roads beside the line, and along lanes across it
    for xr_, z0_, z1_, step in ((MID + 36, FARM + 150, BR0 - 70, 9.0), (MID - 48, BR1 + 40, END + 350, 10.0),
                                (MID + 62, BR1 + 20, END + 250, 9.5)):
        for zz in np.arange(z0_, z1_, step):
            broadleaf(xr_ + float(rng.uniform(-0.8, 0.8)), float(zz + rng.uniform(-1.5, 1.5)),
                      float(rng.uniform(2.6, 3.6)), float(rng.uniform(7, 9.5)))
    for zc_, sgn, x0_, x1_ in ((FARM + 270, -1, 24, 260), (BR1 + 130, 1, 24, 300), (FARM + 120, 1, 60, 300)):
        for xx in np.arange(x0_, x1_, 11.0):
            broadleaf(MID + sgn * (xx + float(rng.uniform(-1, 1))), zc_ + float(rng.uniform(-0.8, 0.8)),
                      float(rng.uniform(2.6, 3.8)), float(rng.uniform(7, 10)))
    for v in range(14):                                   # villages further off, in their groves
        x = MID + float(rng.choice([-1, 1]) * rng.uniform(90, 420))
        z = float(rng.uniform(FARM + 150, END + 900))
        if not clear_of_line(x, z, 60):
            continue
        for k in range(int(rng.integers(4, 9))):
            hx, hz = x + float(rng.uniform(-35, 35)), z + float(rng.uniform(-30, 30))
            house(hx, hz, float(rng.uniform(8, 14)), float(rng.uniform(6, 10)), float(rng.uniform(3.2, 6)),
                  bool(rng.random() < 0.5))
        for k in range(int(rng.integers(4, 9))):
            cedar(x + float(rng.uniform(-45, 45)), z + float(rng.uniform(-40, 40)), float(rng.uniform(12, 20)))
        for k in range(int(rng.integers(3, 6))):
            broadleaf(x + float(rng.uniform(-45, 45)), z + float(rng.uniform(-40, 40)), float(rng.uniform(3, 5)),
                      float(rng.uniform(8, 12)))
    # -- the bridge: a through truss, Warren with verticals, 10 m panels
    tr_m = mats.add(K_TRUSS, (0.62, 0.10, 0.04))
    deck = mats.add(K_CONC, (0.40, 0.40, 0.39), (8.0,))
    g.place(model("truss_bridge"), (0.0, 0.0, BR0), 0.0, roles={"steel": tr_m, "conc": deck})
    return g

POLES = []

NOSE = 10.7

def build_train(mats):
    """The other train from its models: the nose car, fourteen middle cars, the nose car turned round at the
    tail; pantographs on two cars. Its materials by role (K_TRAIN, p[0] the role)."""
    g = Geo()
    roles = {r: mats.add(K_TRAIN, (0.70, 0.70, 0.69), (float(i),))
             for i, r in enumerate(["body", "stripe", "skirt", "winband", "screen", "lamp", "nose"])}
    roles["gangway"] = mats.add(K_DARK, (0.03, 0.03, 0.035))
    for c in range(CARS):
        z0 = c * CAR
        if c == 0:
            g.place(model("n700_nose"), (0.0, 0.0, z0), 0.0, roles=roles)
        elif c == CARS - 1:
            g.place(model("n700_nose"), (0.0, 0.0, z0 + CAR), math.pi, roles=roles)
        else:
            g.place(model("n700_mid"), (0.0, 0.0, z0), 0.0, roles=roles)
        if c in (4, 12):
            g.place(model("pantograph"), (0.0, 0.0, z0 + 12), 0.0, roles={"gangway": roles["gangway"]})
    return g

# ---- the frame ----------------------------------------------------------------------------------------------------

STATIC = {}
# hooks for the night extension (train_night.py); left as they are, the ride renders exactly as without them
HOOKS = {}                                                 # "build", "dynamic", "light", "post", "cab"
NIGHT = [0.0]                                              # how dark it is: lit windows come on with it
OWN_FRONT = [0.0]                                          # our own train's nose, when it's seen from outside
TRAIN_DZ = [-SPEED]                                        # how far the drawn trains move in a frame
WIRE_BACK = [0.0]                                          # draw the overhead line this far behind the camera too
LOW_HAZE = [np.array([0.42, 0.26, 0.40])]                  # the evening's haze away from the sun
TOWN0 = [1e9]                                              # beyond this the land is the town's streets, not paddies
TRIS = [0]                                                 # triangles drawn this frame (all passes)
CUT = np.zeros(20000, bool)                              # materials with cut-out foliage
NOCULL = np.zeros(20000, bool)                           # materials drawn from both sides

def scene():
    if not STATIC:
        mats = Mats()
        g = build(mats)
        if HOOKS.get("build"):
            HOOKS["build"](g, mats)
        V, N, M = g.arrays()
        V = bent(V)
        gt = build_train(mats)
        TV, TN, TM = gt.arrays()
        far = V[..., 2].max(1) > TUN1 - 10
        dusk = SMap(DUSK.sun, (MID + 40, 0.0, 1050.0), -450, 550, -80, 75, 0.15, 0.05)
        dusk.raster(V[far])
        near = V[..., 2].min(1) < TUN0 + 60
        day = SMap(DAY.sun, (MID, 0.0, 60.0), -150, 150, -230, 230, 0.08, 0.08)
        day.raster(V[near])
        CUT[:] = False
        CUT[:len(mats.kind)] = np.array(mats.kind) == K_TREE
        lod, iz_ = g.lod, g.iz
        kinds = np.array(mats.kind)[M]
        STATIC.update(small=np.isin(kinds, SMALL), xmid=V[..., 0].mean(1), lod=lod, iz=iz_)
        STATIC.update(V=V, N=N, M=M, TV=TV, TN=TN, TM=TM, mats=mats.arrays(), smap_dusk=dusk, smap_day=day,
                      zlo=V[..., 2].min(1), zhi=V[..., 2].max(1))
    return STATIC

# the sun's shadow maps: orthographic depth maps looking down the sun's rays, one for the city by day, one for the
# evening half
class SMap:
    def __init__(self, sun, centre, u0, u1, v0, v1, du, dv):
        f = -sun
        self.r = nrm(np.cross([0, 1.0, 0], f)); self.u = np.cross(f, self.r); self.f = f
        self.c = np.asarray(centre, float)
        self.u0, self.v1, self.du, self.dv = u0, v1, du, dv
        self.W, self.H = int((u1 - u0) / du), int((v1 - v0) / dv)
        self.D = np.full((self.H, self.W), np.inf, np.float32)

    def coords(self, P):
        Q = P - self.c
        return (Q @ self.r - self.u0) / self.du, (self.v1 - Q @ self.u) / self.dv, Q @ self.f

    def raster(self, V, D=None):
        D = self.D if D is None else D
        pu, pv, pd = self.coords(V)
        for i in range(len(V)):
            sx, sy, dz = pu[i], pv[i], pd[i]
            x0, x1 = max(0, int(math.floor(sx.min()))), min(self.W, int(math.ceil(sx.max())) + 1)
            y0, y1 = max(0, int(math.floor(sy.min()))), min(self.H, int(math.ceil(sy.max())) + 1)
            if x0 >= x1 or y0 >= y1:
                continue
            d = (sx[1] - sx[0]) * (sy[2] - sy[0]) - (sx[2] - sx[0]) * (sy[1] - sy[0])
            if abs(d) < 1e-9:
                continue
            px = np.arange(x0, x1, dtype=np.float32)[None, :] + 0.5
            py = np.arange(y0, y1, dtype=np.float32)[:, None] + 0.5
            l1 = ((px - sx[0]) * (sy[2] - sy[0]) - (sx[2] - sx[0]) * (py - sy[0])) / d
            l2 = ((sx[1] - sx[0]) * (py - sy[0]) - (px - sx[0]) * (sy[1] - sy[0])) / d
            l0 = 1 - l1 - l2
            e = -0.02
            inside = (l0 >= e) & (l1 >= e) & (l2 >= e)
            if not inside.any():
                continue
            z = l0 * dz[0] + l1 * dz[1] + l2 * dz[2]
            sub = D[y0:y1, x0:x1]
            w = inside & (z < sub)
            sub[w] = z[w]

    def lookup(self, P, mask, D=None, bias=0.35):
        D = self.D if D is None else D
        lit = np.ones(P.shape[:2], np.float32)
        idx = np.nonzero(mask)
        if len(idx[0]) == 0:
            return lit
        pu, pv, pd = self.coords(P[idx])
        acc = np.zeros(len(pu), np.float32)
        n = 0
        for oy in (-1.5, -0.5, 0.5, 1.5):
            for ox in (-1.5, -0.5, 0.5, 1.5):
                ix = np.clip((pu + ox).astype(int), 0, self.W - 1)
                iy = np.clip((pv + oy).astype(int), 0, self.H - 1)
                acc += (D[iy, ix] < pd - bias)
                n += 1
        inmap = (pu > 0) & (pu < self.W) & (pv > 0) & (pv < self.H)
        lit[idx] = np.where(inmap, 1 - acc / n, 1.0)
        return lit

def clip_tri(c, n):
    """A triangle in camera space (with its vertex normals) cut at z = NEAR, as a fan of triangles."""
    out = []
    for k in range(3):
        p, q = c[k], c[(k + 1) % 3]
        np_, nq = n[k], n[(k + 1) % 3]
        pin, qin = p[2] >= NEAR, q[2] >= NEAR
        if pin:
            out.append((p, np_))
        if pin != qin:
            t = (NEAR - p[2]) / (q[2] - p[2])
            out.append((p + (q - p) * t, np_ + (nq - np_) * t))
    return [(np.array([out[0][0], out[k][0], out[k + 1][0]]), np.array([out[0][1], out[k][1], out[k + 1][1]]))
            for k in range(1, len(out) - 1)]

def raster(V, N, M, cam, IZ, MB, NB):
    W, H, f = cam.W, cam.H, cam.f
    eye, B = cam.eye, cam.basis
    # cull: behind, backfacing
    fnorm = np.cross(V[:, 1] - V[:, 0], V[:, 2] - V[:, 0])
    avgn = N.sum(1)
    flip = np.sum(fnorm * avgn, -1) < 0
    fnorm[flip] *= -1
    front = (np.sum(fnorm * (V[:, 0] - eye), -1) < 0) | CUT[M] | NOCULL[M]   # foliage shows its far side
    C = np.einsum('nij,kj->nik', V - eye, B)
    z = C[..., 2]
    keep = front & (z.max(1) > NEAR)
    C, N2, M2 = C[keep], N[keep], M[keep]
    z = C[..., 2]
    allin = z.min(1) >= NEAR
    with np.errstate(divide='ignore', invalid='ignore'):
        sx = W / 2 + f * C[..., 0] / z
        sy = H / 2 - f * C[..., 1] / z
    off = allin & ((sx.max(1) < 0) | (sx.min(1) > W) | (sy.max(1) < 0) | (sy.min(1) > H))
    jobs = []
    for i in np.nonzero(~off)[0]:
        if allin[i]:
            jobs.append((C[i], N2[i], M2[i]))
        else:
            for cc, nn in clip_tri(C[i], N2[i]):
                jobs.append((cc, nn, M2[i]))
    TRIS[0] += len(jobs)
    for c, n, m in jobs:
        zc = c[:, 2]
        sx = W / 2 + f * c[:, 0] / zc
        sy = H / 2 - f * c[:, 1] / zc
        x0, x1 = max(0, int(math.floor(sx.min() - 0.5))), min(W, int(math.ceil(sx.max() + 0.5)))
        y0, y1 = max(0, int(math.floor(sy.min() - 0.5))), min(H, int(math.ceil(sy.max() + 0.5)))
        if x0 >= x1 or y0 >= y1:
            continue
        d = (sx[1] - sx[0]) * (sy[2] - sy[0]) - (sx[2] - sx[0]) * (sy[1] - sy[0])
        if abs(d) < 1e-9:
            continue
        px = np.arange(x0, x1, dtype=np.float32)[None, :] + 0.5
        py = np.arange(y0, y1, dtype=np.float32)[:, None] + 0.5
        l1 = ((px - sx[0]) * (sy[2] - sy[0]) - (sx[2] - sx[0]) * (py - sy[0])) / d
        l2 = ((sx[1] - sx[0]) * (py - sy[0]) - (px - sx[0]) * (sy[1] - sy[0])) / d
        l0 = 1 - l1 - l2
        e = -1e-5
        inside = (l0 >= e) & (l1 >= e) & (l2 >= e)
        if not inside.any():
            continue
        iz = l0 / zc[0] + l1 / zc[1] + l2 / zc[2]
        zb = IZ[y0:y1, x0:x1]
        win = inside & (iz > zb)
        if not win.any():
            continue
        w0, w1, w2 = (l0 / zc[0])[win], (l1 / zc[1])[win], (l2 / zc[2])[win]
        nn = (w0[:, None] * n[0] + w1[:, None] * n[1] + w2[:, None] * n[2])
        if CUT[m]:                                        # foliage: ragged, holes toward the outline
            zz = 1 / iz[win]
            qx = ((px - W / 2) / f + 0 * py)[win]; qy = (-(py - H / 2) / f + 0 * px)[win]
            d = qx[:, None] * B[0] + qy[:, None] * B[1] + B[2]
            P = eye + d * zz[:, None]
            dn = d / np.linalg.norm(d, axis=1, keepdims=True)
            nl = nn / np.maximum(np.linalg.norm(nn, axis=1, keepdims=True), 1e-6)
            facing = np.abs(np.sum(dn * nl, 1))
            sc = 1.7
            lv = vnoise(P[:, 0] * sc + P[:, 1] * 0.8, P[:, 2] * sc - P[:, 1] * 0.6) * 0.65 + \
                vnoise(P[:, 0] * sc * 2.7 - P[:, 1], P[:, 2] * sc * 2.7 + P[:, 1] * 1.3) * 0.35
            keep = lv > 0.12 + 0.75 * (1 - facing) ** 2
            idx = np.nonzero(win)
            win = np.zeros_like(win); win[idx[0][keep], idx[1][keep]] = True
            nn = nn[keep]
            if not win.any():
                continue
        zb[win] = iz[win]
        MB[y0:y1, x0:x1][win] = m
        NB[y0:y1, x0:x1][win] = nn

def ground(cam, Dw, IZ, MB, NB):
    """The ground, analytic: the deck at y = 0 where there is one, else the land (or the street, the river)."""
    eye = cam.eye
    dy = Dw[..., 1]
    down = dy < -1e-6
    with np.errstate(divide='ignore', invalid='ignore'):
        t0 = np.where(down, -eye[1] / dy, np.inf)
    x0, z0 = eye[0] + Dw[..., 0] * t0, eye[2] + Dw[..., 2] * t0
    xr = x0 - bend(z0) - MID
    hw = np.where(z0 < TUN0 - 60, 5.7, np.where(z0 < TUN0, 12.0, np.where(z0 < TUN1, 5.0, np.where(
        (z0 >= BR0) & (z0 < BR1), 5.9, 5.0))))
    on = down & (np.abs(xr) < hw) & (t0 < 4000)
    IZ[on] = 1 / t0[on]; MB[on] = K_DECK; NB[on] = (0, 1, 0)
    rest = down & ~on
    city = cam.z < TUN0 - 20
    yl = STREET if city else LAND
    with np.errstate(divide='ignore', invalid='ignore'):
        t1 = np.where(rest, (yl - eye[1]) / dy, np.inf)
    x1, z1 = eye[0] + Dw[..., 0] * t1, eye[2] + Dw[..., 2] * t1
    t1 = np.minimum(t1, 60000)
    if city:
        IZ[rest] = 1 / t1[rest]; MB[rest] = K_STREET; NB[rest] = (0, 1, 0)
        return
    xr1 = x1 - bend(z1) - MID
    emb = rest & (np.abs(xr1) < 5.0 + 1.5 * 2.0) & ~((z1 > BR0) & (z1 < BR1))
    river = rest & ~emb & (z1 > RIV0 - BANK) & (z1 < RIV1 + BANK)
    with np.errstate(divide='ignore', invalid='ignore'):
        t2 = np.where(river, (RIVER - eye[1]) / dy, np.inf)
    z2 = eye[2] + Dw[..., 2] * t2
    water = river & (z2 > RIV0) & (z2 < RIV1)
    bank = river & ~water
    land = rest & ~emb & ~river
    IZ[land] = 1 / t1[land]; MB[land] = K_PADDY; NB[land] = (0, 1, 0)
    MB[land & (z1 > TOWN0[0])] = K_STREET
    IZ[emb] = 1 / t1[emb]; MB[emb] = K_EMBANK
    s = np.sign(xr1[emb])[:, None]
    NB[emb] = np.concatenate([s * 0.55, np.ones_like(s) * 0.83, np.zeros_like(s)], 1)
    IZ[water] = 1 / t2[water]; MB[water] = K_WATER; NB[water] = (0, 1, 0)
    # the bank: where the land ray dips into the river's trough without reaching the water; a slope
    tb = np.where(bank, t1, np.inf)
    zb = eye[2] + Dw[..., 2] * tb
    near_side = zb < (RIV0 + RIV1) / 2
    # slide the hit down the slope: solve along the ray for y = LAND - (depth into the bank)
    IZ[bank] = 1 / tb[bank]; MB[bank] = K_BANK
    nb = np.where(near_side[bank], 1.0, -1.0)[:, None]
    NB[bank] = np.concatenate([np.zeros_like(nb), np.ones_like(nb) * 0.85, nb * 0.5], 1)

def city_shadow(P, Nn, sun, boxes, mask):
    """1 where the sun reaches, 0 where a building (or a parapet) is in the way: a ray against each box."""
    lit = np.ones(P.shape[:2], np.float32)
    idx = np.nonzero(mask)
    if len(idx[0]) == 0 or len(boxes) == 0:
        return lit
    p = P[idx] + Nn[idx] * 0.05
    inv = 1.0 / np.where(np.abs(sun) < 1e-6, 1e-6, sun)
    hit = np.zeros(len(p), bool)
    for lo, hi in boxes:
        ta, tb = (lo - p) * inv, (hi - p) * inv
        tn = np.minimum(ta, tb).max(1)
        tf = np.maximum(ta, tb).min(1)
        hit |= (tn < tf) & (tf > 0) & (tn < 400)
    lit[idx] = np.where(hit, 0.0, 1.0)
    return lit

def tunnel_light(P, Nn, cam, tunnel):
    """The lamps on both walls every 12 m (the nearest few of each row), and our headlights."""
    L = np.zeros(P.shape, np.float32)
    p, n = P[tunnel], Nn[tunnel]
    acc = np.zeros_like(p)
    dx = math.sqrt(5.6 ** 2 - 0.7 ** 2) - 0.4
    k0 = np.round((p[:, 2] - (TUN0 + 6.65)) / 12.0)
    for xl in (MID - dx, MID + dx):
        for dk in (-2, -1, 0, 1, 2):
            zl = TUN0 + 6.65 + (k0 + dk) * 12.0
            valid = (zl > TUN0) & (zl < TUN1)
            v = np.stack([xl - p[:, 0], 4.2 - p[:, 1], zl - p[:, 2]], 1)
            d2 = np.sum(v * v, 1)
            ndl = np.clip(np.sum(v * n, 1) / np.sqrt(d2), 0, 1)
            acc += (valid * 1.3 * ndl / (d2 + 3.0))[:, None] * np.array([1.0, 0.70, 0.38])
    for hx in (-0.75, 0.75):                              # the headlights, a cone ahead
        hp = cam.eye + np.array([hx, -2.2, 1.5])
        v = hp - p
        d2 = np.sum(v * v, 1)
        dist = np.sqrt(d2)
        l = v / dist[:, None]
        cone = np.clip(-(l @ nrm([0, -0.06, 1])), 0, 1) ** 30
        acc += (60.0 * cone * np.clip(np.sum(l * n, 1), 0, 1) / (d2 + 1))[:, None] * np.array([0.9, 0.95, 1.0])
    L[tunnel] = acc
    return L

def weathering(u, y, top, base, seed, walls):
    """Grime on a wall: rain streaks running down from the top and the sills, dirt splashed up from the ground,
    blotches. A multiplier, 1 = clean."""
    streak = smooth(0.5, 0.85, fbm(u * 1.3 + seed, y * 0.05, 3)) * (0.5 + 0.5 * smooth(0, 6, top - y + 3))
    drip = np.exp(-np.maximum(top - y, 0) / 2.2) * (0.6 + 0.4 * vnoise(u * 2.1 + seed, y * 0.3))
    dirt = np.exp(-np.maximum(y - base, 0) / 1.2)
    blot = smooth(0.55, 0.9, fbm(u * 0.25 + seed, y * 0.25, 2))
    return 1 - walls * np.clip(0.22 * streak + 0.25 * drip + 0.25 * dirt + 0.12 * blot, 0, 0.6)

def schlick(cosv, f0):
    return f0 + (1 - f0) * (1 - np.clip(cosv, 0, 1)) ** 5

def reflect(D, N):
    return D - 2 * np.sum(D * N, -1, keepdims=True) * N

def gbuffer(cam, S, V, N, M, TV, ground_too=True):
    W, H = cam.W, cam.H
    IZ = np.zeros((H, W), np.float32)
    MB = np.zeros((H, W), np.int32)
    NB = np.zeros((H, W, 3), np.float32)
    if ground_too:
        ground(cam, cam.Dw, IZ, MB, NB)
    raster(V, N, M, cam, IZ, MB, NB)
    if TV is not None:
        raster(TV[0], TV[1], TV[2], cam, IZ, MB, NB)
    return IZ, MB, NB

SMALL = (K_TREE, K_WOOD, K_ROOFTOP, K_SOLAR, K_SIGN, K_SHED, K_HOUSE, K_TILE, K_CHOUSE)

def visible(S, cam):
    """The triangles worth sending: ahead within 2.6 km; from the city and inside the tunnel's first half only
    what's this side of the hill (the hill hides the farmland); through the exit only what the portal frames;
    small things (trees, houses, units, signs) only within 900 m."""
    zlo, zhi, small = S["zlo"], S["zhi"], S["small"]
    sel = (zhi > cam.z - 5) & (zlo < cam.z + 2600)
    if cam.z < TUN0 + 120:
        sel &= zlo < TUN0 + 260
    elif cam.z < TUN1:
        sel &= (zhi < TUN1 + 5) | ((zlo < TUN1 + 1200) & (np.abs(S["xmid"] - MID - 30) < 260))
    sel &= ~small | (zlo < cam.z + 900)
    near = S["iz"] < cam.z + 260                           # trees: the full model near, one crown far
    sel &= (S["lod"] == 0) | ((S["lod"] == 1) & near) | ((S["lod"] == 2) & ~near)
    return sel

def mirror_pass(cam, env, t, front, S, V, N, M, TV, h):
    """What the water at height h mirrors: everything above it flipped below it, rendered and shaded as itself."""
    def flip(V, N, M):
        keep = (V[..., 1].max(1) > h + 0.05) & (V[..., 2].min(1) < cam.z + 450)   # far reflections are specks
        V2, N2 = V[keep].copy(), N[keep].copy()
        V2[..., 1] = 2 * h - V2[..., 1]; N2[..., 1] *= -1
        return V2, N2, M[keep]
    TVm = None if TV is None else flip(*TV)
    IZ, MB, NB = gbuffer(cam, S, *flip(V, N, M), TVm, ground_too=False)
    if h == RIVER:                                        # the land beyond the river, upside down in it
        dy = cam.Dw[..., 1]
        with np.errstate(divide="ignore", invalid="ignore"):
            t_ = np.where(dy < -1e-6, (2 * h - LAND - cam.eye[1]) / dy, np.inf)
        z_ = cam.eye[2] + cam.Dw[..., 2] * t_
        land = np.isfinite(t_) & (t_ < 20000) & ((z_ > RIV1) | (z_ < RIV0)) & (1 / t_ > IZ)
        bank = land & (((z_ > RIV1) & (z_ < RIV1 + BANK)) | ((z_ < RIV0) & (z_ > RIV0 - BANK)))
        IZ[land] = 1 / t_[land]; MB[land] = K_PADDY; MB[bank] = K_BANK; NB[land] = (0, -1, 0)
    Z = np.where(IZ > 0, 1 / np.maximum(IZ, 1e-9), 1e6)
    P = cam.eye + cam.Dw * Z[..., None]
    P[..., 1] = 2 * h - P[..., 1]
    NB[..., 1] *= -1
    Nn = NB / np.maximum(np.linalg.norm(NB, axis=-1, keepdims=True), 1e-6)
    Dn = cam.Dn.copy(); Dn[..., 1] *= -1
    img = shade(cam, env, t, front, S, Dn, P, Nn, MB, IZ, Z, TV, None)
    return img, Z, IZ > 0

def render(t, W, H, overlay=True):
    TRIS[0] = 0
    S = scene()
    cam = Cam(t, W, H)
    env = env_at(cam.z)
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32) + 0.5
    cx, cy = (xs - W / 2) / cam.f, -(ys - H / 2) / cam.f
    B = cam.basis
    cam.Dw = cx[..., None] * B[0] + cy[..., None] * B[1] + B[2]
    cam.Dn = cam.Dw / np.linalg.norm(cam.Dw, axis=-1, keepdims=True)
    Dw, Dn = cam.Dw, cam.Dn
    sel = visible(S, cam)
    V, N, M = S["V"][sel], S["N"][sel], S["M"][sel]
    front = train_front(t)
    TV = None
    if front < cam.z + 3000 and front + CARS * CAR > cam.z - 5:
        TV = (bent(S["TV"] + np.array([X2, 0.0, front])), S["TN"], S["TM"])
    if HOOKS.get("dynamic"):
        TV = HOOKS["dynamic"](t, cam, S, TV)
    IZ, MB, NB = gbuffer(cam, S, V, N, M, TV)
    Z = np.where(IZ > 0, 1 / np.maximum(IZ, 1e-9), 1e6)
    P = cam.eye + Dw * Z[..., None]
    Nn = NB / np.maximum(np.linalg.norm(NB, axis=-1, keepdims=True), 1e-6)
    KIND = S["mats"][0]
    kind = KIND[MB]
    sky_m = IZ <= 0
    refl = {}
    if not os.environ.get("NOREFL"):
        for k_, h in ((K_PADDY, LAND), (K_WATER, RIVER)):
            if (kind == k_).sum() > 50:
                refl[k_] = mirror_pass(cam, env, t, front, S, V, N, M, TV, h)
    img = shade(cam, env, t, front, S, Dn, P, Nn, MB, IZ, Z, TV, refl)
    if os.environ.get("STATS"):
        print("frame %.1f triangles drawn %d (mirrored passes: %d)" % (t, TRIS[0], len(refl)), flush=True)
    # -- motion: blur along each pixel's screen velocity over the shutter
    if not os.environ.get("NOBLUR"):
        vel = velocity(cam, P, sky_m, kind, front, t)
        img = motion_blur(img, vel, Z, sky_m)
    img = wires(img, cam, IZ, env)
    if HOOKS.get("post"):
        img = HOOKS["post"](img, cam, IZ, Z, t)
    img = img * exposure(cam.z)
    r2 = ((xs - W / 2) ** 2 + (ys - H / 2) ** 2) / ((W / 2) ** 2 + (H / 2) ** 2)
    img = img * (1 - 0.22 * r2)[..., None]                # a little vignette, the lens
    img = flare(img, cam, env, sky_m, xs, ys)
    img = bloom(img, cam)
    out = tonemap(img)
    if overlay and not os.environ.get("NOTEXT"):
        out = overlays(out, t, W, H)
    return out


def shade(cam, env, t, front, S, Dn, P, Nn, MB, IZ, Z, TV, refl):
    """Light and colour every pixel from its position, normal and material; the sky where there's nothing."""
    H, W = IZ.shape
    KIND, ALB, PAR = S["mats"]
    Nn = Nn.copy()
    mirrors = refl or {}
    kind = KIND[MB]
    sky_m = IZ <= 0
    img = np.zeros((H, W, 3), np.float32)
    img[sky_m] = sky(Dn[sky_m], cam.eye, env)
    alb = ALB[MB].copy()
    emit = np.zeros_like(img)
    spec = np.zeros_like(img)                             # reflected light, added after the diffuse
    rough = np.zeros((H, W), np.float32)
    fpx = Z / cam.f                                       # a pixel's width at that depth, in metres
    cosv = np.abs(np.sum(Dn * Nn, -1))
    foot = fpx / np.maximum(cosv, 0.08)                   # and its footprint on the surface, roughly
    pr = PAR[MB]
    X, Y, Zw = P[..., 0], P[..., 1], P[..., 2]
    xrel = X - bend(Zw)                                   # across the line
    def refl_sky(mask, Nm, tint, f0, gloss=1.0):
        d = Dn[mask]; n = Nm
        r = reflect(d, n)
        r[:, 1] = np.abs(r[:, 1]) * 0.7 + 0.3 * r[:, 1]
        r /= np.linalg.norm(r, axis=-1, keepdims=True)
        F = schlick(np.abs(np.sum(d * n, -1)), f0)
        return sky(r, cam.eye, env) * (F * gloss)[:, None] * np.asarray(tint)

    def mirrored(m, k_, r, nx, nz, amp):
        """The sky in the mirrored direction, or what stands above the water, from the mirror pass, shifted by
        the ripples."""
        col = sky(r, cam.eye, env)
        if k_ in mirrors:
            rimg, rZ, rhas = mirrors[k_]
            yy, xx = np.nonzero(m)
            sx = np.clip((xx + nx * amp).astype(int), 0, W - 1)
            sy = np.clip((yy - nz * amp).astype(int), 0, H - 1)
            obj = rhas[sy, sx] & (rZ[sy, sx] > Z[m])
            col = np.where(obj[:, None], rimg[sy, sx], col)
        return col

    # -- the deck: slab track (city, tunnel, bridge) or ballast (the farmland), the four rails
    m = kind == K_DECK
    if m.any():
        xr, z = xrel[m] , Zw[m]
        fx, fz = fpx[m], fpx[m] * Z[m] / EYE_Y
        ballast = (z > TUN1) & ~((z > BR0) & (z < BR1))
        n = fbm(xr * 6, z * 6, 2)
        conc = np.array([0.30, 0.295, 0.285]) * (0.85 + 0.3 * n)[:, None]
        gravel = np.array([0.30, 0.27, 0.24]) * (0.6 + 0.8 * fbm(xr * 18, z * 18, 2))[:, None]
        base = np.where(ballast[:, None], gravel, conc)
        col = base.copy()
        for c0 in (0.0, X2):
            slab = fband(xr, c0, 1.17, fx) * (1 - fstripe(z, 4.95, 0.03, fz) * 0.8)
            slabc = np.array([0.22, 0.22, 0.215]) * (0.9 + 0.2 * n)[:, None]
            sleeper = fband(xr, c0, 1.25, fx) * fstripe(z + 0.1, 0.6, 0.38, fz)
            slpc = np.array([0.55, 0.54, 0.52])
            col = np.where(ballast[:, None], col * (1 - sleeper[:, None]) + slpc * sleeper[:, None],
                           col * (1 - slab[:, None]) + slabc * slab[:, None])
            for rx in (c0 - 0.7175, c0 + 0.7175):
                dark = fband(xr, rx, 0.11, fx)
                col = col * (1 - 0.75 * dark[:, None])
        cut = (np.abs(xr - MID) > 6.2) & (z > TUN0 - 60) & (z < TUN0)
        verge = np.array([0.11, 0.15, 0.06]) * (0.6 + 0.8 * fbm(xr * 1.5, z * 1.5, 3))[:, None]
        col = np.where(cut[:, None], verge, col)
        alb[m] = col
        rail = np.zeros(len(xr), np.float32)
        for c0 in (0.0, X2):
            for rx in (c0 - 0.7175, c0 + 0.7175):
                rail = np.maximum(rail, fband(xr, rx, 0.034, fx))
        r = Dn[m].copy(); r[:, 1] = -r[:, 1]
        rs = sky(r, cam.eye, env) if cam.z > TUN0 + 10 or cam.z < TUN0 - 5 else np.zeros_like(r)
        intun = (z > TUN0) & (z < TUN1)
        rs[intun] = 0
        spec[m] = rs * (rail * 0.85)[:, None]
        alb[m] = alb[m] * (1 - rail[:, None]) + np.array([0.6, 0.6, 0.62]) * rail[:, None] * intun[:, None]
    # -- the paddies: water mirroring the sky between the seedlings, dikes, the farm road, the embankment
    m = kind == K_PADDY
    if m.any():
        x, z = X[m], Zw[m]
        xr = xrel[m] - MID
        fx = foot[m]
        cxp, czp = (x + 7.0) / 31.0, (z + 3.0) / 19.0
        ix, iz = np.floor(cxp), np.floor(czp)
        u, v = (cxp - ix) * 31.0, (czp - iz) * 19.0
        dike = np.maximum(fband(u, 0.55, 0.55, fx) + fband(u, 30.45, 0.55, fx),
                          fband(v, 0.55, 0.55, fx) + fband(v, 18.45, 0.55, fx))
        dike = np.clip(dike, 0, 1)
        h = hash2(ix, iz)
        flooded = h < 0.78
        grown = (h >= 0.78) & (h < 0.92)
        ripple_t = t * 0.15
        nx = (fbm(x * 0.35 + ripple_t, z * 0.12, 2) - 0.5) * 0.035
        nz = (fbm(x * 0.12 - 3.1, z * 0.35 - ripple_t, 2) - 0.5) * 0.035
        wn = np.stack([nx, np.ones_like(nx), nz], 1); wn /= np.linalg.norm(wn, axis=1, keepdims=True)
        d = Dn[m]
        r = reflect(d, wn); r[:, 1] = np.abs(r[:, 1])
        F = schlick(-d[:, 1], 0.02)
        refl = mirrored(m, K_PADDY, r, nx, nz, 0.3 * W) * F[:, None]
        rowc = fstripe(u + 0.07 * np.sin(v * 0.4), 0.30, 0.22, fx) * fstripe(v, 0.18, 0.55, fx * Z[m] / 6)
        rows = np.where(flooded, rowc * 0.9, 0.0)
        mud = np.array([0.10, 0.08, 0.05])
        seed = np.array([0.10, 0.22, 0.04]) * (0.8 + 0.4 * h)[:, None]
        grn = np.array([0.12, 0.26, 0.05]) * (0.7 + 0.6 * fbm(x * 0.8, z * 0.8, 2))[:, None]
        soil = np.array([0.20, 0.14, 0.09]) * (0.8 + 0.4 * fbm(x * 2, z * 2, 2))[:, None]
        grass = np.array([0.13, 0.20, 0.05]) * (0.75 + 0.5 * fbm(x * 1.5, z * 1.5, 2))[:, None]
        a = np.where(flooded[:, None], mud * (1 - rows[:, None]) + seed * rows[:, None],
                     np.where(grown[:, None], grn, soil))
        road = fband(xr, -19.0 + 2.6, 2.2, fx)
        a = a * (1 - dike[:, None]) + grass * dike[:, None]
        a = a * (1 - road[:, None]) + np.array([0.33, 0.30, 0.26]) * road[:, None]
        wet = flooded * (1 - dike) * (1 - rows) * (1 - road)
        alb[m] = a * np.where(flooded[:, None], 1.0 - wet[:, None] * 0.6, 1.0)
        spec[m] = refl * wet[:, None]
    m = kind == K_EMBANK
    if m.any():
        alb[m] = np.array([0.14, 0.21, 0.06]) * (0.7 + 0.6 * fbm(X[m] * 1.2, Zw[m] * 1.2, 3))[:, None]
    m = kind == K_BANK
    if m.any():
        alb[m] = np.array([0.16, 0.18, 0.08]) * (0.6 + 0.7 * fbm(X[m] * 0.9, Zw[m] * 2.5, 3))[:, None]
    m = kind == K_STREET
    if m.any():
        alb[m] = np.array([0.12, 0.12, 0.13])
    # -- the river: wide, slow ripples, the sun's path on it
    m = kind == K_WATER
    if m.any():
        x, z = X[m], Zw[m]
        tt = t * 0.25
        nx = (fbm(x * 0.08 + tt, z * 0.5, 3) - 0.5) * 0.14 + (vnoise(x * 0.6, z * 2.5 + tt * 3) - 0.5) * 0.05
        nz = (fbm(x * 0.5 - 2.0, z * 0.08 - tt, 3) - 0.5) * 0.10
        wn = np.stack([nx, np.ones_like(nx), nz], 1); wn /= np.linalg.norm(wn, axis=1, keepdims=True)
        d = Dn[m]
        r = reflect(d, wn); r[:, 1] = np.abs(r[:, 1]) + 0.002
        r /= np.linalg.norm(r, axis=1, keepdims=True)
        F = schlick(-np.sum(d * wn, 1), 0.02)
        refl = mirrored(m, K_WATER, r, nx, nz, 0.15 * W)
        glint = np.clip(r @ env.sun, 0, 1) ** 900 * 40 + np.clip(r @ env.sun, 0, 1) ** 90 * 1.5
        spec[m] = refl * F[:, None] + (glint * F * 3)[:, None] * env.sun_col
        alb[m] = np.array([0.03, 0.05, 0.05])
    # -- buildings: windows by floor, glass mirroring the sky
    m = kind == K_BUILD
    if m.any():
        n = Nn[m]; p = P[m]
        side = np.abs(n[:, 0]) > 0.5
        top = n[:, 1] > 0.5
        u = np.where(side, p[:, 2], p[:, 0])
        v = p[:, 1] - STREET
        fw = foot[m]
        style, pitch, gk, rnd = pr[m][:, 0], pr[m][:, 1], pr[m][:, 2], pr[m][:, 3]
        fl = 3.6
        winv = fstripe(v - 1.0, fl, 0.55, fw)
        winu = fstripe(u + rnd * 3, pitch, 0.62, fw)
        mull = fstripe(u + rnd * 3, 1.6, 0.92, fw)
        glassf = np.where(style == 0, winv * winu, np.where(style == 1, winv * mull, fstripe(v, fl, 0.88, fw) * mull))
        balc = fstripe(v + 0.2, fl, 0.32, fw)                             # apartments: balcony fronts, doors
        glassf = np.where(style == 3, (1 - balc) * fstripe(u + rnd * 3, pitch * 1.4, 0.8, fw), glassf)
        glassf = np.where(top, 0.0, glassf)
        glassf = np.where(v < 9.0, glassf * 0.3, glassf)
        a = alb[m] * (0.88 + 0.24 * vnoise(u * 0.05 + rnd * 9, v * 0.1))[:, None]
        a = np.where(top[:, None], np.array([0.38, 0.38, 0.37]), a)
        gtint = np.array([(0.55, 0.75, 0.85), (0.50, 0.70, 0.60), (0.75, 0.62, 0.48), (0.62, 0.70, 0.80)])[
            gk.astype(int)]
        wh = hash2(np.floor(u / pitch) + rnd * 31, np.floor(v / fl) + rnd * 17)   # each window its own
        inner = np.array([0.025, 0.03, 0.035]) * (0.6 + 0.8 * wh)[:, None]
        blind = ((wh > 0.72) & (style != 2)).astype(np.float32)
        frac_b = np.where(wh > 0.86, 1.0, 0.55)                                    # blinds half or all down
        vin = np.mod(v - 1.0, fl) / (fl * 0.55)
        blind = blind * (vin > 1 - frac_b)
        inner = inner * (1 - blind[:, None]) + np.array([0.42, 0.40, 0.35]) * blind[:, None]
        lintel = np.clip((vin - 0.8) / 0.12, 0, 1) * (style != 2)          # the recess's shadow under the lintel
        inner = inner * (1 - 0.6 * lintel[:, None])
        alb[m] = a * (1 - glassf[:, None]) + inner * glassf[:, None]
        alb[m] = alb[m] * weathering(u, p[:, 1], pr[m][:, 4], pr[m][:, 5], rnd * 50, np.where(top, 0.0, 1.0))[:, None]
        rs = refl_sky(m, n, 1.0, 0.25) * (1 - 0.7 * lintel[:, None])
        spec[m] = rs * gtint * (glassf * (0.75 + 0.5 * wh) * (1 - 0.6 * blind))[:, None]
        on = hash2(np.floor(u / pitch) * 1.7 + rnd * 13, np.floor(v / fl) * 2.3) < NIGHT[0] * 0.7
        emit[m] += np.where((style == 2)[:, None], np.array([0.75, 0.85, 1.0]), np.array([1.0, 0.82, 0.58])) * \
            (glassf * on * (1.0 + 0.6 * wh))[:, None]
    m = kind == K_ROOFTOP
    if m.any():
        alb[m] = alb[m] * (0.8 + 0.4 * vnoise(X[m] * 0.5, Zw[m] * 0.5))[:, None]
    m = kind == K_SIGN
    if m.any():
        p = P[m]; n = Nn[m]
        u = np.where(np.abs(n[:, 0]) > 0.5, p[:, 2], p[:, 0])
        vert = pr[m][:, 1] < 0.5                          # a vertical sign: characters stacked; a board: in a row
        a_ = np.where(vert, p[:, 1], u) / 1.25
        b_ = np.where(vert, u, p[:, 1]) / 1.25
        ci, fa = np.floor(a_), a_ - np.floor(a_)
        fb = b_ - np.floor(b_)
        cs = ci + pr[m][:, 0] * 13.0 + np.floor(b_) * 71.0       # each character its own strokes
        fwc = foot[m] / 1.25
        glyph = np.zeros(len(fa), np.float32)
        for k, yb in enumerate((0.18, 0.5, 0.82)):         # horizontal strokes
            on = hash2(cs, k * 1.7) > 0.4
            x0_, x1_ = 0.12 + 0.3 * hash2(cs, k + 9.1), 0.58 + 0.3 * hash2(cs, k + 4.3)
            glyph = np.maximum(glyph, on * fband(fa, yb, 0.06, fwc) * fband(fb, (x0_ + x1_) / 2, (x1_ - x0_) / 2, fwc))
        for k, xb in enumerate((0.28, 0.5, 0.72)):         # vertical strokes
            on = hash2(cs, k * 2.9 + 20) > 0.45
            y0_, y1_ = 0.1 + 0.35 * hash2(cs, k + 31.7), 0.55 + 0.35 * hash2(cs, k + 17.3)
            glyph = np.maximum(glyph, on * fband(fb, xb, 0.06, fwc) * fband(fa, (y0_ + y1_) / 2, (y1_ - y0_) / 2, fwc))
        face = np.abs(n[:, 1]) < 0.5
        glyph = glyph * face
        base = alb[m]
        ink = np.where((base.sum(1) > 2.5)[:, None], np.array([0.6, 0.05, 0.05]), np.array([1.0, 1.0, 0.95]))
        alb[m] = base * (1 - glyph[:, None]) + ink * glyph[:, None]
        emit[m] = alb[m] * (0.25 + 2.5 * NIGHT[0])
    # -- concrete: parapets, the portal, piers; joints every few metres
    m = kind == K_CONC
    if m.any():
        p = P[m]; n = Nn[m]
        u = np.where(np.abs(n[:, 0]) > 0.5, p[:, 2], p[:, 0])
        jl = pr[m][:, 0]
        j = fstripe(u, np.maximum(jl, 0.5), 0.02, foot[m])
        stain = 0.8 + 0.35 * fbm(u * 0.4, p[:, 1] * 0.9, 3)
        alb[m] = alb[m] * stain[:, None] * (1 - 0.5 * j[:, None])
    m = kind == K_HILL
    if m.any():                                           # a wooded hillside: crowns as bumps, lit from the sun
        p = P[m]
        def crowns(x, y, z):
            return fbm(x * 0.28 + y * 0.1, z * 0.28 - y * 0.1, 3)
        h0 = crowns(p[:, 0], p[:, 1], p[:, 2])
        e = 0.6
        hx = (crowns(p[:, 0] + e, p[:, 1], p[:, 2]) - h0) / e
        hz = (crowns(p[:, 0], p[:, 1], p[:, 2] + e) - h0) / e
        bump = np.stack([-hx * 9, np.zeros_like(hx), -hz * 9], 1)
        Nn[m] = Nn[m] + bump
        Nn[m] /= np.linalg.norm(Nn[m], axis=1, keepdims=True)
        gaps = smooth(0.25, 0.55, h0)
        alb[m] = alb[m] * (0.35 + 1.1 * gaps)[:, None] * (0.8 + 0.4 * vnoise(p[:, 0] * 0.05, p[:, 2] * 0.05))[:, None]
    m = kind == K_TUNNEL
    if m.any():
        p = P[m]
        ring = fstripe(p[:, 2] - TUN0, 10.5, 0.015, foot[m])
        stain = 0.7 + 0.5 * fbm(p[:, 2] * 0.15, p[:, 1] * 0.6 + p[:, 0] * 0.3, 3)
        alb[m] = alb[m] * stain[:, None] * (1 - 0.6 * ring[:, None])
    m = kind == K_LAMP
    if m.any():
        emit[m] = np.array([1.0, 0.72, 0.38]) * 30.0
        alb[m] = 0
    m = (kind == K_METAL) | (kind == K_TRUSS)
    if m.any():
        spec[m] = refl_sky(m, Nn[m], 0.5, 0.06)
    m = kind == K_TREE
    if m.any():
        p = P[m]
        away = np.sum(Nn[m] * Dn[m], 1) > 0                              # the far side, seen through the holes
        Nn[m] = np.where(away[:, None], -Nn[m] * 0.6 + np.array([0, 0.4, 0]), Nn[m])
        sp = pr[m][:, 0]
        sc = np.where(sp == 1, 1.6, np.where(sp == 3, 2.2, 0.9))
        lv = fbm(p[:, 0] * sc + p[:, 1] * 0.7, p[:, 2] * sc - p[:, 1] * 0.5, 3)
        tint = 0.85 + 0.3 * vnoise(p[:, 0] * 0.05 + pr[m][:, 1], p[:, 2] * 0.05)
        alb[m] = alb[m] * ((0.45 + 1.2 * lv) * tint * np.where(away, 0.55, 1.0))[:, None]
        Nn[m] = Nn[m] + np.stack([lv - 0.5, np.zeros_like(lv), 0.5 - lv], 1) * 0.8
        Nn[m] /= np.linalg.norm(Nn[m], axis=1, keepdims=True)
        back = np.clip(Dn[m] @ env.sun, 0, 1) ** 6                      # backlit: light through the leaves' edges
        rim = (1 - np.abs(np.sum(Dn[m] * Nn[m], 1))) ** 2
        emit[m] = alb[m] * env.sun_col * (back * rim * (0.4 + lv) * 1.6)[:, None]
    m = kind == K_CHOUSE
    if m.any():                                           # the suburb's houses: siding, sashes, wear
        p = P[m]; n = Nn[m]
        seed, top, base, pitch, sty = pr[m][:, 0], pr[m][:, 1], pr[m][:, 2], pr[m][:, 3], pr[m][:, 4]
        u = np.where(np.abs(n[:, 0]) > 0.5, p[:, 2], p[:, 0])
        v = p[:, 1] - base
        fw = foot[m]
        cellu, cellv = np.floor((u + seed) / pitch), np.floor(v / 2.85)
        hs = hash2(cellu + seed, cellv)
        win = fstripe(u + seed + pitch * 0.2, pitch, 0.45, fw) * fband(np.mod(v, 2.85), 1.55, 0.55, fw) * (hs > 0.35)
        win = win * (v < top - base - 0.3)
        lines = np.where(sty == 1, fstripe(v, 0.21, 0.12, fw), np.where(sty == 2, fstripe(u, 0.45, 0.08, fw), 0.0))
        a = alb[m] * (1 - 0.18 * lines[:, None])
        frame = np.array([0.55, 0.56, 0.58])
        glass = np.array([0.03, 0.035, 0.04]) * (0.6 + 0.9 * hs)[:, None]
        curtain = (hs > 0.75)[:, None] * np.array([0.40, 0.36, 0.30])
        a = a * (1 - win[:, None]) + (glass + curtain) * win[:, None]
        rail = fband(np.mod(v, 2.85), 0.95, 0.06, fw) * (hs > 0.6) * (v > 2.0)       # a balcony rail here and there
        a = a * (1 - rail[:, None]) + frame * rail[:, None]
        alb[m] = a * weathering(u, p[:, 1], top, base, seed, 1.0)[:, None]
        spec[m] = refl_sky(m, n, 0.8, 0.12) * (win * (1 - (hs > 0.75)))[:, None]
        on = hash2(cellu * 3.1 + seed, cellv + 7.7) < NIGHT[0] * 0.75                    # lights coming on
        emit[m] += np.where((hs > 0.75)[:, None], np.array([1.0, 0.62, 0.30]), np.array([1.0, 0.80, 0.55])) * \
            (win * on * 1.4)[:, None]
    m = kind == K_SOLAR
    if m.any():                                           # solar panels: dark cells, a silver grid, the sky in them
        p = P[m]
        grid = np.maximum(fstripe(p[:, 0], 1.0, 0.04, foot[m]), fstripe(p[:, 2], 1.65, 0.03, foot[m]))
        alb[m] = alb[m] * (1 - grid[:, None]) + np.array([0.5, 0.5, 0.52]) * grid[:, None]
        spec[m] = refl_sky(m, Nn[m], 0.8, 0.08) * (1 - grid)[:, None]
    m = kind == K_HOUSE
    if m.any():
        p = P[m]; n = Nn[m]
        u = np.where(np.abs(n[:, 0]) > 0.5, p[:, 2], p[:, 0])
        v = p[:, 1] - LAND
        post = fstripe(u, 1.82, 0.08, foot[m])
        lower = smooth(1.1, 0.95, v)
        wood = np.array([0.22, 0.14, 0.08])
        k = np.clip(post + lower, 0, 1)
        win = fstripe(u + 0.4, 3.64, 0.5, foot[m]) * fband(v, 1.9, 0.6, foot[m])
        a = alb[m] * (1 - k[:, None]) + wood * k[:, None]
        alb[m] = a * (1 - win[:, None]) + np.array([0.04, 0.04, 0.05]) * win[:, None]
        on = hash2(np.floor((u + 0.4) / 3.64) + pr[m][:, 0] * 5, p[:, 0] * 0.01) < NIGHT[0] * 0.8
        emit[m] += np.array([1.0, 0.75, 0.45]) * (win * on * 1.3)[:, None]
        spec[m] = refl_sky(m, n, 0.6, 0.1) * win[:, None]
    m = kind == K_TILE
    if m.any():
        p = P[m]
        rows = fstripe(p[:, 1] * 3.0 + 0.0, 1.0, 0.25, foot[m] * 3)
        alb[m] = alb[m] * (1 - 0.4 * rows[:, None])
        spec[m] = refl_sky(m, Nn[m], 0.7, 0.08)
    m = kind == K_SHED
    if m.any():
        p = P[m]; n = Nn[m]
        u = np.where(np.abs(n[:, 0]) > 0.5, p[:, 2], p[:, 0])
        cor = fstripe(u, 0.2, 0.5, foot[m])
        rust = smooth(0.5, 0.8, fbm(p[:, 0] * 0.4, p[:, 2] * 0.4 + p[:, 1], 3))
        alb[m] = alb[m] * (0.75 + 0.3 * cor)[:, None] * (1 - 0.3 * rust[:, None]) + \
            np.array([0.12, 0.05, 0.02]) * rust[:, None]
    m = kind == K_WOOD
    if m.any():
        alb[m] = alb[m] * (0.8 + 0.4 * vnoise(P[m][:, 1] * 3, P[m][:, 0] * 3))[:, None]
    # -- the other train: white paint with a clear coat, the blue stripes, the windows, its lamps
    m = kind == K_TRAIN
    if m.any():                                           # the other train: white paint with a clear coat
        p = P[m]; n = Nn[m]
        role = pr[m][:, 0]
        own = pr[m][:, 1] > 0.5                                  # our own train, its nose toward +z
        lz = np.where(own, OWN_FRONT[0] - p[:, 2], p[:, 2] - front)
        cz = np.mod(lz, CAR)
        car = np.floor(lz / CAR)
        fw = foot[m]
        white = np.array([0.70, 0.70, 0.69])
        a = np.tile(white, (len(role), 1))
        a = np.where((role == 1)[:, None], np.array([0.02, 0.10, 0.42]), a)            # the blue stripes
        a = np.where((role == 2)[:, None], white * 0.55, a)                              # the skirt
        dn = np.where(car == 0, cz, CAR - cz)                                            # down the nose, per pixel
        sy = 1.15 - 0.62 * (1 - smooth(0.5, NOSE, dn))
        yy = p[:, 1]
        ns = np.clip(fband(yy, sy, 0.16, fw) + fband(yy, sy + 0.30, 0.036, fw), 0, 1) * (role == 6)
        a = a * (1 - ns[:, None]) + np.array([0.02, 0.10, 0.42]) * ns[:, None]
        a = np.where(((role == 6) & (yy < 0.55))[:, None], a * 0.6, a)
        s_ = np.clip(dn / NOSE, 0, 1)                                                    # the canopy's crease
        hwn = 1.69 * (0.60 + 0.40 * (1 - (1 - s_) ** 2.4))
        lx = p[:, 0] - bend(p[:, 2]) - X2
        crease = np.exp(-((np.abs(lx) / (0.46 * hwn) - 0.95) / 0.18) ** 2) * (role == 6) * (s_ > 0.3) * (s_ < 0.85)
        a = a * (1 - 0.45 * crease[:, None])
        win = (role == 3) * fstripe(cz + 0.3, 1.05, 0.72, fw) * (cz > 1.5) * (cz < CAR - 1.5) * \
            (car > 0) * (car < CARS - 1) + (role == 3) * ((car == 0) | (car == CARS - 1)) * \
            fstripe(cz + 0.3, 1.05, 0.72, fw) * (np.minimum(cz, CAR - cz) > 1.5) * \
            np.where(car == 0, cz > 13, CAR - cz > 13)
        dark = np.clip(win + (role == 4), 0, 1)
        a = a * (1 - dark[:, None]) + np.array([0.015, 0.02, 0.025]) * dark[:, None]
        alb[m] = a
        rs = refl_sky(m, n, 1.0, 0.045)
        grey = rs.mean(1, keepdims=True)
        rs = np.where((role == 4)[:, None], (rs * 0.3 + grey * 0.7) * 0.18, rs)       # the windscreen: dark glass
        spec[m] = rs * (1 + 1.0 * win[:, None])
        emit[m] += np.array([1.0, 0.85, 0.6]) * (win * NIGHT[0] * 1.6)[:, None]       # lit inside at night
        tail = car == CARS - 1
        lampc = np.where(tail[:, None], np.array([3.0, 0.1, 0.05]), np.array([4.0, 3.8, 3.4]))
        emit[m] = lampc * (role == 5)[:, None] * 6.0
        back = np.clip(Dn[m] @ env.sun, 0, 1) ** 3                       # the low sun behind it: a rim of light
        rim = (1 - np.abs(np.sum(Dn[m] * n, 1))) ** 3 * (1 - dark) * smooth(0.2, 0.7, n[:, 1])
        emit[m] += env.sun_col * (back * rim * float(os.environ.get("RIM", "0.35")))[:, None]
    m = kind == K_DARK
    if m.any():
        alb[m] = np.array([0.03, 0.03, 0.035])
    m = kind == K_METAL
    if m.any():
        alb[m] = alb[m] * (0.85 + 0.3 * vnoise(P[m][:, 1] * 2, P[m][:, 2]))[:, None]

    if HOOKS.get("materials"):
        HOOKS["materials"](kind, P, Nn, Dn, pr, alb, emit, spec, foot, t)
    # -- light it: the sun (with the city's shadows), the sky's ambient, the tunnel's lamps
    solid = ~sky_m
    tunnel = solid & (Zw > TUN0 - 0.5) & (Zw < TUN1 + 0.5) & (np.abs(xrel - MID) < 5.75) & (Y < 9.2)
    ndl = np.clip(np.sum(Nn * env.sun, -1), 0, 1)
    sh = np.ones((H, W), np.float32)
    if env is DAY:
        mask = solid & (ndl > 0) & (Zw < TUN0 + 2) & ~tunnel
        sh = S["smap_day"].lookup(P + Nn * 0.3, mask)
    else:
        sm = S["smap_dusk"]
        D = sm.D
        if TV is not None:
            D = D.copy()
            sm.raster(TV[0], D)
        mask = solid & (ndl > 0) & ~tunnel & (Zw > TUN1 - 1)
        sh = sm.lookup(P + Nn * 0.25, mask, D)
    hemi = (0.5 + 0.5 * Nn[..., 1])[..., None]
    amb = env.amb * hemi + env.bounce * (1 - hemi)
    if env is DAY:                                        # the street canyon darker low down
        canyon = (0.45 + 0.55 * smooth(STREET, 40.0, Y))[..., None]
        amb = np.where(((kind == K_BUILD) | (kind == K_SIGN))[..., None], amb * canyon, amb)
    light = env.sun_col * (ndl * sh)[..., None] + amb
    # in the tunnel: no sun, the sky's light only near the mouths
    dmouth = np.minimum(np.abs(Zw - TUN0), np.abs(TUN1 - Zw))
    leak = np.exp(-dmouth / 12.0)[..., None]
    mouth_amb = np.where((Zw < (TUN0 + TUN1) / 2)[..., None], DAY.amb * 0.8, DUSK.amb * 1.6)
    tl = tunnel_light(P, Nn, cam, tunnel)
    light = np.where(tunnel[..., None], tl + mouth_amb * leak + 0.006, light)
    if HOOKS.get("light"):
        light = light + HOOKS["light"](P, Nn, cam, solid & ~tunnel, kind)
    spec = np.where(tunnel[..., None] & (kind != K_DECK)[..., None], spec * leak, spec)
    col = alb * light + spec + emit
    img[solid] = col[solid]
    # aerial perspective
    dist = Z * np.linalg.norm(cam.Dw, axis=-1)
    T = np.exp(-dist / env.fog)[..., None]
    if env is DAY:
        haze = env.haze
    else:
        sunw = np.clip(Dn @ env.sun, 0, 1)[..., None] ** 4
        haze = LOW_HAZE[0] * (1 - sunw) + env.haze * sunw
    fogm = solid[..., None] & ~tunnel[..., None]
    img = np.where(fogm, img * T + haze * (1 - T), img)
    return img

def flare(img, cam, env, sky_m, xs, ys):
    """Ghosts of the low sun in the lens, on the line from the sun through the middle, as much as the sun shows."""
    if env is not DUSK:
        return img
    W, H = cam.W, cam.H
    sx, sy, sz = cam.project(cam.eye + env.sun * 1e5)
    sx, sy = float(sx), float(sy)
    if sz <= 0 or not (-0.1 * W < sx < 1.1 * W and -0.1 * H < sy < 1.1 * H):
        return img
    r = max(2, int(0.012 * W))
    x0, x1 = int(np.clip(sx - r, 0, W - 1)), int(np.clip(sx + r + 1, 1, W))
    y0, y1 = int(np.clip(sy - r, 0, H - 1)), int(np.clip(sy + r + 1, 1, H))
    vis = float(sky_m[y0:y1, x0:x1].mean()) if x1 > x0 and y1 > y0 else 0.0
    if vis <= 0.01:
        return img
    cx, cy = W / 2, H / 2
    for k, rad, col in ((0.45, 0.025, (0.9, 0.5, 0.2)), (0.8, 0.012, (0.4, 0.8, 0.5)), (1.25, 0.05, (0.35, 0.3, 0.8)),
                        (1.6, 0.018, (0.9, 0.4, 0.6)), (2.1, 0.09, (0.5, 0.35, 0.25))):
        gx, gy = sx + (cx - sx) * k, sy + (cy - sy) * k
        R = rad * W
        d = np.hypot(xs - gx, ys - gy)
        disc = (1 - smooth(0.75 * R, R, d)) * (0.6 + 0.4 * smooth(0.0, R, d))
        img = img + disc[..., None] * np.array(col) * 0.06 * vis * float(env.sun_col[0] / 2.6)
    return img

def velocity(cam, P, sky_m, kind, front, t):
    a, b = Cam(t - SHUTTER / 2, cam.W, cam.H), Cam(t + SHUTTER / 2, cam.W, cam.H)
    Pa, Pb = P.copy(), P.copy()
    farP = cam.eye + (P - cam.eye) / np.maximum(np.linalg.norm(P - cam.eye, axis=-1, keepdims=True), 1e-6) * 1e5
    Pa[sky_m] = farP[sky_m] - cam.eye + a.eye
    Pb[sky_m] = farP[sky_m] - cam.eye + b.eye
    tr = (kind == K_TRAIN) | (kind == K_DARK)
    Pa[tr, 2] -= TRAIN_DZ[0] * SHUTTER / 2
    Pb[tr, 2] += TRAIN_DZ[0] * SHUTTER / 2
    if HOOKS.get("vel"):
        HOOKS["vel"](Pa, Pb, kind, t)
    xa, ya, za = a.project(Pa)
    xb, yb, zb = b.project(Pb)
    v = np.stack([xb - xa, yb - ya], -1)
    bad = (za < NEAR) | (zb < NEAR)
    v[bad] = 0
    mag = np.linalg.norm(v, axis=-1, keepdims=True)
    lim = 0.09 * cam.W
    v = v * np.minimum(1, lim / np.maximum(mag, 1e-6))
    return v.astype(np.float32)

def motion_blur(img, vel, Z, sky_m, taps=28):
    H, W = img.shape[:2]
    T = 16
    th, tw = (H + T - 1) // T, (W + T - 1) // T
    mag = np.linalg.norm(vel, axis=-1)
    padm = np.zeros((th * T, tw * T), np.float32); padm[:H, :W] = mag
    padv = np.zeros((th * T, tw * T, 2), np.float32); padv[:H, :W] = vel
    tm = padm.reshape(th, T, tw, T).transpose(0, 2, 1, 3).reshape(th, tw, T * T)
    am = tm.argmax(-1)
    tv = padv.reshape(th, T, tw, T, 2).transpose(0, 2, 1, 3, 4).reshape(th, tw, T * T, 2)
    tile = np.take_along_axis(tv, am[..., None, None], 2)[:, :, 0]
    tmag = np.linalg.norm(tile, axis=-1)
    nb = tile.copy(); nm = tmag.copy()
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            sh = np.roll(np.roll(tile, dy, 0), dx, 1)
            shm = np.roll(np.roll(tmag, dy, 0), dx, 1)
            better = shm > nm
            nb[better] = sh[better]; nm[better] = shm[better]
    Vn = np.repeat(np.repeat(nb, T, 0), T, 1)[:H, :W]
    Vm = np.linalg.norm(Vn, axis=-1)
    if Vm.max() < 0.7:
        return img
    ys, xs = np.mgrid[0:H, 0:W]
    ign = (52.9829189 * ((0.06711056 * xs + 0.00583715 * ys) % 1.0)) % 1.0   # interleaved gradient noise: even,
    jit = (ign - 0.5).astype(np.float32)                                      # not grainy
    Zc = np.where(sky_m, 1e6, Z).astype(np.float32)
    rp = np.maximum(mag, 0.5)
    soft = 0.04 * Zc + 0.5
    wsum = 1.0 / rp
    acc = img * wsum[..., None]
    for j in range(taps):
        tj = (j // 2 * 2 + 1 + jit * 2) / taps - 0.5            # alternate taps: the neighbourhood's, the pixel's own
        Vd = Vn if j % 2 == 0 else vel
        sx = np.clip(np.round(xs + Vd[..., 0] * tj), 0, W - 1).astype(np.int32)
        sy = np.clip(np.round(ys + Vd[..., 1] * tj), 0, H - 1).astype(np.int32)
        dist = np.abs(tj) * (Vm if j % 2 == 0 else mag)
        zs, rs = Zc[sy, sx], np.maximum(mag[sy, sx], 0.5)
        fr = np.clip(1 - (zs - Zc) / soft, 0, 1)           # the sample is in front
        bk = np.clip(1 - (Zc - zs) / soft, 0, 1)           # the sample is behind
        cone_s = np.clip(1 - dist / rs, 0, 1)
        cone_p = np.clip(1 - dist / rp, 0, 1)
        cyl = (1 - smooth(0.95 * rs, 1.05 * rs, dist)) * (1 - smooth(0.95 * rp, 1.05 * rp, dist))
        w = fr * cone_s + bk * cone_p + cyl * 2
        acc += img[sy, sx] * w[..., None]
        wsum = wsum + w
    return acc / wsum[..., None]

def wires(img, cam, IZ, env):
    """The overhead line: the contact wire zig-zagging between the masts, the messenger above it sagging, the
    droppers; the farm road's power lines. Anti-aliased lines, behind what's in front of them."""
    H, W = img.shape[:2]
    lines = []
    z0 = cam.z + 1.0
    zs = np.arange(math.floor((z0 - WIRE_BACK[0]) / 5) * 5 + 5, cam.z + 700, 5.0)
    zs = np.concatenate([[z0], zs]) if not WIRE_BACK[0] else zs
    intun = lambda z: (z > TUN0) & (z < TUN1)
    for xw in (0.0, X2):
        span = np.mod(zs, 50.0) / 50.0
        stagger = 0.22 * (1 - 2 * np.abs(np.mod(zs / 50.0, 2) - 1))
        cy = np.full_like(zs, 5.1)
        my = 6.6 - 0.9 * 4 * span * (1 - span)
        my = np.where(intun(zs), 5.9, my)
        P1 = np.stack([xw + stagger - 0.11 + bend(zs), cy, zs], 1)
        P2 = np.stack([xw + bend(zs), my, zs], 1)
        lines.append((P1, 0.012, 0.02))
        lines.append((P2, 0.010, 0.015))
        for k in range(len(zs)):                          # droppers every 5 m
            if zs[k] < cam.z + 120:
                lines.append((np.array([P1[k], P2[k]]), 0.006, 0.02))
    if cam.z > TUN1 - 40:                                 # the power lines on the farm road's poles
        for zp0, zp1 in zip(POLES[:-1], POLES[1:]):
            if zp1 < cam.z or zp0 > cam.z + 600:
                continue
            zz = np.linspace(zp0, zp1, 9)
            u = (zz - zp0) / (zp1 - zp0)
            for dx in (-1.1, 0.0, 1.1):
                yy = LAND + 9.85 - 0.9 * 4 * u * (1 - u)
                lines.append((np.stack([np.full_like(zz, MID - 19 + dx) + bend(zz), yy, zz], 1), 0.012, 0.02))
    if cam.z < TUN0:                                       # the suburb's wires
        for P, rad in CITY_WIRES:
            if P[-1, 2] > cam.z + 1 and P[0, 2] < cam.z + 450:
                lines.append((P, rad, 0.0))
    col_dark = np.array([0.02, 0.02, 0.02])
    for P, rad, shine in lines:
        sx, sy, sz = cam.project(P)
        for i in range(len(P) - 1):
            if sz[i] < NEAR or sz[i + 1] < NEAR:
                continue
            ax, ay, bx, by = sx[i], sy[i], sx[i + 1], sy[i + 1]
            wa, wb = max(rad * cam.f / sz[i], 0.35), max(rad * cam.f / sz[i + 1], 0.35)
            x0 = max(0, int(min(ax, bx) - wa - 2)); x1 = min(W, int(max(ax, bx) + wa + 3))
            y0 = max(0, int(min(ay, by) - wa - 2)); y1 = min(H, int(max(ay, by) + wa + 3))
            if x0 >= x1 or y0 >= y1:
                continue
            px = np.arange(x0, x1, dtype=np.float32)[None, :] + 0.5
            py = np.arange(y0, y1, dtype=np.float32)[:, None] + 0.5
            dx, dy = bx - ax, by - ay
            L2 = dx * dx + dy * dy + 1e-9
            u = np.clip(((px - ax) * dx + (py - ay) * dy) / L2, 0, 1)
            d = np.hypot(px - (ax + u * dx), py - (ay + u * dy))
            w = wa + (wb - wa) * u
            iz = 1 / (sz[i] + (sz[i + 1] - sz[i]) * u)
            cov = np.clip(w + 0.5 - d, 0, 1) * np.minimum(1, 2 * w)
            cov = cov * (iz >= IZ[y0:y1, x0:x1] * 0.995)
            if not cov.any():
                continue
            sub = img[y0:y1, x0:x1]
            c = col_dark + (env.sun_col * shine if env is DUSK else 0)
            sub[:] = sub * (1 - cov[..., None]) + c * cov[..., None]
    return img

def box_blur(a, r, axis):
    c = np.cumsum(a, axis=axis, dtype=np.float32)
    n = a.shape[axis]
    pad = [(0, 0)] * a.ndim; pad[axis] = (1, 0)
    c = np.pad(c, pad)
    idx = np.arange(n)
    hi = np.clip(idx + r + 1, 0, n); lo = np.clip(idx - r, 0, n)
    out = (np.take(c, hi, axis) - np.take(c, lo, axis)) / (hi - lo).reshape([-1 if k == axis else 1 for k in range(a.ndim)])
    return out

def gblur(a, r):
    for _ in range(3):
        a = box_blur(box_blur(a, r, 0), r, 1)
    return a

def upsample(a, H, W):
    h, w = a.shape[:2]
    ys = (np.arange(H) + 0.5) * h / H - 0.5
    xs = (np.arange(W) + 0.5) * w / W - 0.5
    y0 = np.clip(np.floor(ys).astype(int), 0, h - 1); y1 = np.clip(y0 + 1, 0, h - 1)
    x0 = np.clip(np.floor(xs).astype(int), 0, w - 1); x1 = np.clip(x0 + 1, 0, w - 1)
    fy = np.clip(ys - y0, 0, 1)[:, None, None]; fx = np.clip(xs - x0, 0, 1)[None, :, None]
    r0 = a[y0][:, x0] * (1 - fx) + a[y0][:, x1] * fx
    r1 = a[y1][:, x0] * (1 - fx) + a[y1][:, x1] * fx
    return r0 * (1 - fy) + r1 * fy

def bloom(img, cam):
    H, W = img.shape[:2]
    k = 4
    small = img[:H // k * k, :W // k * k].reshape(H // k, k, W // k, k, 3).mean((1, 3))
    br = np.maximum(small - 1.3, 0)
    s = H / 768
    b1 = gblur(br, max(1, int(3 * s)))
    b2 = gblur(br, max(1, int(12 * s)))
    b = b1 * 0.5 + b2 * 0.5
    return img + upsample(b, H, W) * 0.4

def tonemap(x):
    x = np.maximum(x, 0)
    a = x * (2.51 * x + 0.03) / (x * (2.43 * x + 0.59) + 0.14)
    a = np.clip(a, 0, 1) ** (1 / 2.2)
    a = a ** 1.18                                         # a deeper toe: more contrast in the middle
    g = a.mean(-1, keepdims=True)
    a = np.clip(g + (a - g) * 1.15, 0, 1)                 # and a little more colour
    return a * 255

# ---- the words: the brag and the shader (traceRayEXT as GLSL_EXT_ray_tracing declares it) -------------------------------------------------------

CODE = [
    "#version 460",
    "#extension GL_EXT_ray_tracing : require",
    "layout(binding = 0) uniform accelerationStructureEXT scene;",
    "layout(binding = 1, rgba8) uniform image2D frame;",
    "layout(binding = 2) uniform Camera { vec3 eye, fwd, right, up; } cam;",
    "layout(location = 0) rayPayloadEXT vec3 hit;",
    "",
    "void main() {",
    "    vec2 p = (vec2(gl_LaunchIDEXT.xy) + 0.5) / vec2(gl_LaunchSizeEXT.xy) * 2.0 - 1.0;",
    "    vec3 dir = normalize(cam.fwd + p.x * cam.right - p.y * cam.up);",
    "    traceRayEXT(scene, gl_RayFlagsOpaqueEXT, 0xFFu, 0u, 0u, 0u,",
    "                cam.eye, 0.001, dir, 10000.0, 0);",
    "    imageStore(frame, ivec2(gl_LaunchIDEXT.xy), vec4(hit, 1.0));",
    "}",
]
BRAG = ["REAL-TIME RAY TRACING", "1536 x 768"]
_TXT = {}

def overlays(img, t, W, H):
    from PIL import Image, ImageDraw, ImageFont
    s = W / 1536
    code_lv = float(np.clip(min((t - 4) / 4, (34 - t) / 8), 0, 1))
    brag_lv = float(np.clip(min((t - 14) / 3, (44 - t) / 4), 0, 1))
    if code_lv <= 0 and brag_lv <= 0:
        return img
    key = (W, H)
    if key not in _TXT:
        lay = Image.new("L", (W, H), 0)
        d = ImageDraw.Draw(lay)
        f = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", int(19 * s))
        for i, line in enumerate(CODE):
            d.text((int(70 * s), int(70 * s) + int(27 * s) * i), line, font=f, fill=255)
        fs = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", int(13 * s))
        code = np.array(lay, np.float32) / 255
        lay2 = Image.new("L", (W, H), 0)
        d2 = ImageDraw.Draw(lay2)
        fb = ImageFont.truetype("/System/Library/Fonts/HelveticaNeue.ttc", int(44 * s), index=1)
        y = H - int(170 * s)
        for i, line in enumerate(BRAG):
            d2.text((int(70 * s), y + int(54 * s) * i), line, font=fb, fill=255)
        brag = np.array(lay2, np.float32) / 255
        _TXT[key] = (code, brag)
    code, brag = _TXT[key]
    out = img.astype(np.float32)
    for ink, lv in ((code, code_lv), (brag, brag_lv)):      # a dark glass panel behind the words
        if lv <= 0:
            continue
        ys_, xs_ = np.nonzero(ink > 0.3)
        y0, y1 = max(0, ys_.min() - int(22 * s)), min(H, ys_.max() + int(22 * s))
        x0, x1 = max(0, xs_.min() - int(26 * s)), min(W, xs_.max() + int(26 * s))
        out[y0:y1, x0:x1] *= 1 - 0.55 * lv
    for ink, lv in ((code, code_lv), (brag, brag_lv)):
        if lv <= 0:
            continue
        sh = np.roll(np.roll(ink, 2, 0), 2, 1)
        out = out * (1 - sh[..., None] * 0.75 * lv)
        out = out + (255 - out) * (ink[..., None] * lv)
    return out

# ---- running it ---------------------------------------------------------------------------------------------------

SS = int(os.environ.get("SS", "1"))                      # supersampling: render SS times the size, average down

def frame_bytes(args):
    k, W, H = args
    img = render(float(k), W * SS, H * SS, overlay=False)
    if SS > 1:
        img = img.reshape(H, SS, W, SS, 3).mean((1, 3))
    if not os.environ.get("NOTEXT"):
        img = overlays(img, float(k), W, H)
    return np.clip(img, 0, 255).astype(np.uint8).tobytes()

def main():
    args = sys.argv[1:]
    def opt(name, default):
        if name in args:
            i = args.index(name); v = args[i + 1]; del args[i:i + 2]; return v
        return default
    size = opt("--size", "1536x768")
    W, H = map(int, size.split("x"))
    jobs = int(opt("--jobs", "10"))
    stills = opt("--stills", None)
    rng_ = opt("--frames", None)
    if stills:
        from PIL import Image
        outdir = args[0]
        os.makedirs(outdir, exist_ok=True)
        ks = [float(k) for k in stills.split(",")]
        from multiprocessing import Pool
        with Pool(min(jobs, len(ks))) as pool:
            for k, b in zip(ks, pool.imap(frame_bytes, [(k, W, H) for k in ks])):
                Image.fromarray(np.frombuffer(b, np.uint8).reshape(H, W, 3)).save(f"{outdir}/f{k:06.1f}.png")
        return
    out = args[0]
    a, b = (0, NFRAMES)
    if rng_:
        a, b = map(int, rng_.split(":"))
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}",
                           "-r", str(FPS), "-i", "-", "-c:v", "libx264", "-profile:v", "main", "-level", "4.0",
                           "-bf", "0", "-g", "12", "-crf", "17", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out],
                          stdin=subprocess.PIPE)
    from multiprocessing import Pool
    with Pool(jobs) as pool:
        for i, bts in enumerate(pool.imap(frame_bytes, [(k, W, H) for k in range(a, b)])):
            ff.stdin.write(bts)
            if i % 12 == 0:
                print("frame", a + i, flush=True)
    ff.stdin.close(); ff.wait()
    print(b - a, "frames,", round((b - a) / FPS, 1), "s")

if __name__ == "__main__":
    main()
