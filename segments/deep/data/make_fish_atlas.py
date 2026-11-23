#!/usr/bin/env python3
"""The dive's fish atlas: input data, made once here, like the UI kit's glyphs and the ending's photo. A silver
schooling fish (a mackerel's build: a tapered fusiform body, a forked tail, two dorsal fins and an anal fin, finlets,
pectoral and pelvic fins, an eye, the gill cover) is built as smooth parametric surfaces, bent by the swimming wave
that runs down the body to the tail, and rendered orthographically at a set of yaws, swim phases and elevations.
Each frame is drawn 4 x 4 times over (a z-buffer of surface samples) and averaged, so its edges are antialiased.

Lit as in full light from the surface: light from above, the back dark blue-green, the flanks silver, the belly
pale (countershading); the flanks mirror the bright water above. The mirror's part where the flank turns to the
viewer is kept apart as the flash map, so the machines can add it as a school turns (the script's flash).

    python make_fish_atlas.py      writes fish_atlas.png (RGBA, straight alpha), fish_flash.png (grey) and
                                   fish_atlas.txt (the index) beside it

The layout: cells CELL_W x CELL_H; row = elevation index * len(YAWS) + yaw index, column = phase. The fish's middle
(halfway from snout to tail tip) is the cell's middle; its snout-to-tail length, 1.1 units as in the script, is
PX_PER_UNIT * 1.1 pixels. Yaw 0 is broadside, heading right; positive yaws head away from the viewer, negative
toward; a fish heading left is a yaw frame mirrored. Elevation is how far the viewer is above the fish (degrees).
"""
import os

import numpy as np
from PIL import Image

here = os.path.dirname(os.path.abspath(__file__))
CELL_W, CELL_H = 128, 64
LENGTH = 1.1                                   # snout to tail tip, units (the script's head-to-tail)
PX_PER_UNIT = 112 / LENGTH                     # the fish 112 pixels long broadside
YAWS = list(range(-90, 91, 15))
PHASES = 8
ELEVATIONS = [0, 30]
SS = 4

SNOUT, PEDUNCLE, TAIL = 0.55, -0.33, -0.55     # x along the body; the head +x
H, HP = 0.104, 0.017                           # the body's half height at its deepest, at the peduncle


def smoothstep(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3 - 2 * t)


def tparam(x):
    """Snout 0 to tail tip 1."""
    return (SNOUT - x) / LENGTH


def wave(x, phase):
    """The swimming wave's sideways offset at x: small at the head, growing to the tail, one body length long; the
    head's recoil against the tail's swing."""
    t = tparam(x)
    amp = 0.010 + 0.085 * t ** 2.2
    return amp * np.sin(2 * np.pi * t / 0.95 - phase) - 0.008 * np.sin(-phase)


def half_height(s):
    """The body's half height at s (snout 0 .. peduncle 1)."""
    nose = H * np.sqrt(np.clip(1 - ((0.30 - s) / 0.30) ** 2, 0, 1)) ** 1.1
    back = HP + (H - HP) * (0.5 * (1 + np.cos(np.pi * np.clip((s - 0.32) / 0.64, 0, 1)))) ** 0.8
    return np.where(s < 0.30, nose, back)


def body(phase, nu=900, nv=720):
    s = np.linspace(0, 1, nu)[:, None]
    v = np.linspace(-np.pi, np.pi, nv)[None, :]
    x = SNOUT + (PEDUNCLE - SNOUT) * s + 0 * v
    h = half_height(s)
    w = h * (0.60 - 0.12 * smoothstep(0.6, 1.0, s))
    n = 2.3                                    # a superellipse, a little fuller than an ellipse
    sv, cv = np.sin(v), np.cos(v)
    yrel = np.sign(sv) * np.abs(sv) ** (2 / n)
    zrel = np.sign(cv) * np.abs(cv) ** (2 / n)
    y = np.where(yrel > 0, 1.06, 0.94) * h * yrel - 0.004 * s
    z = w * zrel + wave(x, phase)
    p = np.stack([x, y, z], -1)
    du = np.gradient(p, axis=0)
    dv = np.gradient(p, axis=1)
    nrm = np.cross(du, dv)
    nrm[0] = [1, 0, 0]
    nrm /= np.linalg.norm(nrm, axis=-1, keepdims=True) + 1e-12
    nrm *= np.sign(np.sum(nrm * (p - np.stack([x, 0 * y, wave(x, phase)], -1)), -1, keepdims=True) + 1e-9)
    return p, nrm, s + 0 * v, yrel + 0 * s, zrel + 0 * s


def fin(points, phase, rays=0.0):
    """A thin fin from a grid of (x, y, z offset) and a ray coordinate; the wave carries it with the body."""
    x, y, zo, rc = points
    p = np.stack([x, y, zo + wave(x, phase)], -1)
    du = np.gradient(p, axis=0)
    dv = np.gradient(p, axis=1)
    nrm = np.cross(du, dv)
    nrm /= np.linalg.norm(nrm, axis=-1, keepdims=True) + 1e-12
    stripes = 1.0 if rays == 0 else 0.78 + 0.22 * np.cos(rays * rc)
    return p, nrm, stripes


def caudal():
    a = np.linspace(0, 1, 260)[:, None]
    b = np.linspace(-1, 1, 360)[None, :]
    x0 = PEDUNCLE + 0.015
    length = 0.08 + (x0 - TAIL - 0.08) * np.abs(b) ** 1.25
    x = x0 - a * length
    y = b * (HP * 1.2 + a * 0.17 * (0.30 + 0.70 * np.abs(b) ** 0.7))
    return x, y, 0 * x, b * 14


def dorsal_like(x_front, x_back, height, sweep, top, nu=160, nc=110):
    """A fin standing on the body's back (top=1) or belly (top=-1), tall at its front and swept back."""
    a = np.linspace(0, 1, nu)[:, None]
    c = np.linspace(0, 1, nc)[None, :]
    xb = x_front + (x_back - x_front) * a
    s = (SNOUT - xb) / (SNOUT - PEDUNCLE)
    hb = half_height(np.clip(s, 0, 1)) * (1.06 if top > 0 else 0.94) * 0.97
    hgt = height * (1 - a) ** 0.55 * (0.35 + 0.65 * (1 - a))
    x = xb - c * sweep * (1 - 0.5 * a)
    y = top * (hb + c * hgt)
    return x, y, 0 * x, a * 9


def finlets(x_front, x_back, n, top):
    xs, ys, zs, rs = [], [], [], []
    for k in range(n):
        f = x_front + (x_back - x_front) * k / max(n - 1, 1)
        x, y, z, r = dorsal_like(f, f - 0.022, 0.016, 0.012, top, 30, 24)
        xs.append(x); ys.append(y); zs.append(z); rs.append(r)
    return [np.concatenate(v, 0) for v in (xs, ys, zs, rs)]


def paddle(x_root, y_root, side, length, width, down, out, nu=150, nv=70):
    """A pectoral or pelvic fin: a teardrop from its root on the body's side, pointing back, a little down and out."""
    a = np.linspace(0, 1, nu)[:, None]
    b = np.linspace(-1, 1, nv)[None, :]
    half = width * np.sin(np.pi * np.clip(a, 0, 1) ** 0.6) * (1 - 0.4 * a)
    along, across = a * length, b * half
    d = np.array([-np.cos(down) * np.cos(out), -np.sin(down), side * np.sin(out)])
    e = np.array([-np.sin(down), np.cos(down), 0.0])
    x = x_root + along * d[0] + across * e[0]
    y = y_root + along * d[1] + across * e[1]
    s = (SNOUT - x_root) / (SNOUT - PEDUNCLE)
    zo = side * half_height(np.array(s)) * 0.58 + along * d[2] + across * e[2]
    return x, y, zo, b * 6


def rot(yaw, elev):
    a, e = np.radians(yaw), np.radians(elev)
    ry = np.array([[np.cos(a), 0, np.sin(a)], [0, 1, 0], [-np.sin(a), 0, np.cos(a)]])
    rx = np.array([[1, 0, 0], [0, np.cos(e), -np.sin(e)], [0, np.sin(e), np.cos(e)]])
    return rx @ ry


ENV_TOP = np.array([1.0, 1.0, 1.0])


def env(r):
    """The water as a mirror sees it: bright above, deep blue-green below."""
    k = smoothstep(-0.55, 0.75, r[..., 1])[..., None]
    return np.array([0.20, 0.32, 0.38]) * (1 - k) + np.array([0.96, 1.0, 1.0]) * k


def shade_body(p, nrm, s, yrel, zrel, view, light):
    x = p[..., 0]
    t = tparam(x)
    # countershading: the back, a blue-green with darker wavy bars, the silver flanks, the pale belly
    backc = np.array([0.13, 0.25, 0.30])
    barc = np.array([0.05, 0.11, 0.15])
    flank = np.array([0.80, 0.84, 0.86])
    belly = np.array([0.97, 0.97, 0.95])
    bars = 0.6 * smoothstep(0.2, 0.8, np.sin(48 * x + 16 * yrel + 2.5 * np.sin(31 * x + 5 * yrel)))
    back = backc * (1 - 0.8 * bars[..., None]) + barc * 0.8 * bars[..., None]
    kb = smoothstep(0.36, 0.62, yrel)[..., None]
    kl = smoothstep(-0.05, -0.55, yrel)[..., None]
    alb = flank * (1 - kb) + back * kb
    alb = alb * (1 - kl) + belly * kl
    # an iridescent band where back meets flank
    band = np.exp(-((yrel - 0.30) / 0.06) ** 2)[..., None]
    alb = alb + band * np.array([-0.05, 0.06, 0.10])
    metal = ((0.72 * (1 - kb) + 0.12 * kb) * (1 - 0.35 * kl))[..., 0]
    # the lateral line, the gill cover's edge, the mouth
    lat = np.exp(-((yrel - 0.18 - 0.1 * t) / 0.025) ** 2) * smoothstep(0.15, 0.25, t) * 0.18
    op_x = SNOUT - LENGTH * (0.165 + 0.035 * yrel ** 2)
    op = np.exp(-((x - op_x) / 0.0035) ** 2) * smoothstep(0.75, 0.3, np.abs(yrel)) * 0.45
    mouth = np.exp(-((yrel + 0.12 - 0.3 * t) / 0.04) ** 2) * smoothstep(0.045, 0.025, t) * 0.55
    alb = alb * (1 - (lat + op + mouth)[..., None])
    nl = np.sum(nrm * light, -1)
    nv = np.sum(nrm * view, -1)
    diffuse = 0.50 + 0.50 * np.clip(nl * 0.8 + 0.2, 0, 1)
    r = 2 * nv[..., None] * nrm - view
    mirror = env(r)
    spec = np.clip(np.sum(r * light, -1), 0, 1) ** 40
    col = alb * diffuse[..., None] * (1 - metal[..., None]) + metal[..., None] * mirror * (0.55 + 0.45 * alb)
    col = col + 0.35 * spec[..., None]
    # the edge darkens a little against the water, which keeps the silhouette crisp
    col = col * (0.78 + 0.22 * smoothstep(0.0, 0.45, np.abs(nv)))[..., None]
    # the eye
    ex, ey = SNOUT - 0.072, 0.20
    de = np.hypot((x - ex), (p[..., 1] - ey * H)) / 0.0165
    side_ok = (np.abs(zrel) > 0.3)
    iris = (de < 1.0) & side_ok
    pupil = (de < 0.58) & side_ok
    col = np.where(iris[..., None], np.array([0.80, 0.74, 0.55]) * (0.6 + 0.4 * mirror), col)
    col = np.where(pupil[..., None], np.array([0.02, 0.02, 0.03]) + 0.8 * spec[..., None] + 0.12 * mirror * smoothstep(0.58, 0.2, de)[..., None] * 0, col)
    glint = pupil & (np.hypot(x - ex - 0.004, p[..., 1] - ey * H - 0.005) < 0.0045)
    col = np.where(glint[..., None], np.array([0.95, 0.97, 1.0]), col)
    # the flash: the flank's mirror where it turns to the viewer, kept apart
    flash = metal * (1 - kl[..., 0] * 0.5) * np.clip(nv, 0, 1) ** 5 * smoothstep(0.1, 0.25, t) * (1 - iris)
    return np.clip(col, 0, 1), flash


def shade_fin(nrm, stripes, view, light, tint, alpha):
    nv = np.sum(nrm * view, -1)
    nrm = nrm * np.sign(nv + 1e-9)[..., None]
    nl = np.abs(np.sum(nrm * light, -1))
    col = tint * (0.55 + 0.45 * nl)[..., None]
    a = alpha * (stripes if np.ndim(stripes) else np.ones(nl.shape))
    a = a * (0.75 + 0.25 * (1 - np.abs(np.sum(nrm * view, -1))))   # thicker looking edge-on
    return col, np.clip(a, 0, 1)


def splat(img_w, img_h, scr, depth, colour, extra):
    """Nearest-sample z-buffer: each supersample keeps the nearest surface sample over it."""
    sx = np.round(scr[:, 0]).astype(int)
    sy = np.round(scr[:, 1]).astype(int)
    ok = (sx >= 0) & (sy >= 0) & (sx < img_w) & (sy < img_h)
    sx, sy, depth, colour, extra = sx[ok], sy[ok], depth[ok], colour[ok], extra[ok]
    order = np.argsort(depth)                  # far first; the nearest written last wins
    idx = (sy * img_w + sx)[order]
    zb = np.full(img_w * img_h, -1e9)
    cb = np.zeros((img_w * img_h, colour.shape[1]))
    eb = np.zeros((img_w * img_h, extra.shape[1]))
    zb[idx] = depth[order]
    cb[idx] = colour[order]
    eb[idx] = extra[order]
    return zb, cb, eb


def render(yaw, elev, phase):
    w, h = CELL_W * SS, CELL_H * SS
    scale = PX_PER_UNIT * SS
    R = rot(yaw, elev)
    view = R.T @ np.array([0.0, 0.0, 1.0])     # toward the viewer, in the fish's frame
    light = np.array([0.15, 1.0, 0.25]); light /= np.linalg.norm(light)

    def project(p):
        q = p.reshape(-1, 3) @ R.T
        return np.stack([w / 2 + q[:, 0] * scale, h / 2 - q[:, 1] * scale], -1), q[:, 2]

    p, nrm, s, yrel, zrel = body(phase)
    col, fl = shade_body(p, nrm, s, yrel, zrel, view, light)
    scr, dz = project(p)
    zb, cb, eb = splat(w, h, scr, dz, col.reshape(-1, 3), fl.reshape(-1, 1))
    body_cov = zb > -1e8

    fins = [
        (caudal(), 22, np.array([0.34, 0.42, 0.45]), 0.9),
        (dorsal_like(SNOUT - 0.36, SNOUT - 0.52, 0.075, 0.035, 1), 9, np.array([0.22, 0.30, 0.33]), 0.75),
        (dorsal_like(SNOUT - 0.62, SNOUT - 0.69, 0.030, 0.018, 1), 9, np.array([0.30, 0.36, 0.38]), 0.8),
        (dorsal_like(SNOUT - 0.62, SNOUT - 0.69, 0.028, 0.018, -1), 9, np.array([0.70, 0.72, 0.72]), 0.7),
        (finlets(SNOUT - 0.72, SNOUT - 0.82, 5, 1), 0, np.array([0.30, 0.36, 0.38]), 0.85),
        (finlets(SNOUT - 0.72, SNOUT - 0.82, 5, -1), 0, np.array([0.62, 0.65, 0.66]), 0.8),
        (paddle(SNOUT - 0.215, -0.012, 1, 0.115, 0.022, 0.35, 0.35), 6, np.array([0.55, 0.60, 0.62]), 0.62),
        (paddle(SNOUT - 0.215, -0.012, -1, 0.115, 0.022, 0.35, 0.35), 6, np.array([0.55, 0.60, 0.62]), 0.62),
        (paddle(SNOUT - 0.33, -0.085, 1, 0.045, 0.010, 0.9, 0.25), 4, np.array([0.80, 0.80, 0.78]), 0.6),
        (paddle(SNOUT - 0.33, -0.085, -1, 0.045, 0.010, 0.9, 0.25), 4, np.array([0.80, 0.80, 0.78]), 0.6),
    ]
    fz = np.full(w * h, -1e9); fc = np.zeros((w * h, 3)); fa = np.zeros(w * h)
    for pts, rays, tint, alpha in fins:
        fp, fn, stripes = fin(pts, phase, rays)
        c, a = shade_fin(fn, stripes, view, light, tint, alpha)
        sc, d = project(fp)
        z1, c1, a1 = splat(w, h, sc, d, c.reshape(-1, 3), np.broadcast_to(a, d.shape[:1] if a.ndim == 0 else a.shape).reshape(-1, 1))
        a1 = a1[:, 0]
        nearer = (z1 > fz) & (z1 > -1e8)
        fz = np.where(nearer, z1, fz); fc[nearer] = c1[nearer]; fa = np.where(nearer, a1, fa)

    # fins in front of the body lie over it; behind it, the body hides them
    front = (fz > zb) & (fz > -1e8)
    cov = body_cov.astype(float)
    out_c = cb * cov[:, None]
    out_a = cov.copy()
    over = front & body_cov
    out_c[over] = out_c[over] * (1 - fa[over, None]) + fc[over] * fa[over, None]
    alone = front & ~body_cov
    out_c[alone] = fc[alone] * fa[alone, None]
    out_a[alone] = fa[alone]
    flash = eb[:, 0] * cov
    flash[over] *= (1 - fa[over])

    def down(a):
        a = a.reshape(h // SS, SS, w // SS, SS, *a.shape[2:])
        return a.mean(axis=(1, 3))
    pc, pa, pf = down(out_c.reshape(h, w, 3)), down(out_a.reshape(h, w)), down(flash.reshape(h, w))
    rgb = np.where(pa[..., None] > 1e-4, pc / np.maximum(pa[..., None], 1e-4), 0)
    return rgb, pa, pf


def main():
    rows = len(ELEVATIONS) * len(YAWS)
    atlas = np.zeros((rows * CELL_H, PHASES * CELL_W, 4), np.uint8)
    flash = np.zeros((rows * CELL_H, PHASES * CELL_W), np.uint8)
    for ei, e in enumerate(ELEVATIONS):
        for yi, y in enumerate(YAWS):
            for ph in range(PHASES):
                rgb, a, f = render(y, e, 2 * np.pi * ph / PHASES)
                r0, c0 = (ei * len(YAWS) + yi) * CELL_H, ph * CELL_W
                atlas[r0:r0 + CELL_H, c0:c0 + CELL_W, :3] = np.round(np.clip(rgb, 0, 1) * 255)
                atlas[r0:r0 + CELL_H, c0:c0 + CELL_W, 3] = np.round(np.clip(a, 0, 1) * 255)
                flash[r0:r0 + CELL_H, c0:c0 + CELL_W] = np.round(np.clip(f, 0, 1) * 255)
            print("elevation", e, "yaw", y, flush=True)
    Image.fromarray(atlas, "RGBA").save(os.path.join(here, "fish_atlas.png"), optimize=True)
    Image.fromarray(flash, "L").save(os.path.join(here, "fish_flash.png"), optimize=True)
    with open(os.path.join(here, "fish_atlas.txt"), "w") as f:
        f.write("# the dive's fish atlas: see make_fish_atlas.py\n")
        f.write(f"cell {CELL_W} {CELL_H}\n")
        f.write(f"length_px {round(PX_PER_UNIT * LENGTH)} length_units {LENGTH}\n")
        f.write("anchor middle\n")
        f.write("yaws " + " ".join(map(str, YAWS)) + "\n")
        f.write("elevations " + " ".join(map(str, ELEVATIONS)) + "\n")
        f.write(f"phases {PHASES}\n")
        f.write("layout row = elevation * yaws + yaw; column = phase\n")


if __name__ == "__main__":
    main()
