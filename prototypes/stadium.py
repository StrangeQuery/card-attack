"""Design prototype: the stadium party. Not built on CARDIAC; a picture of the look.

    python stadium.py prep                 # bake the dancers and the stadium in Blender (once; into WORK)
    python stadium.py film OUT.mp4 [--small] [--frames A:B] [--procs N]
    python stadium.py still T OUT.png      # one frame at T seconds
    python stadium.py sheet OUT.png        # the design's stills (outside, the sweep down, the party, close in)
    python stadium.py compare OUT.png      # models against photo cut-outs, at 1536x768 and 384x192

65,536 people (the 16-bit number) in a stadium at night, dancing to the beat: the giant screen at the north end
playing a montage of the demos (a different one every 2 beats), LED walls either side, moving spots through haze, searchlights up into
the sky, laser fans over the crowd, a sea of phone lights, pyro on the stage's edge, confetti. The camera arrives
from outside and high (the train's ending flies in over the station roof and joins here: see `camera()`), sweeps
down over the canopy into the bowl and glides over the crowd toward the stage.

The people are always seen from behind (or far off) as silhouettes with a rim of light: Quaternius's CC0
characters (Poly Pizza), posed by procedural dance moves in Blender (`audience/pose_people.py`: 12 phases of a
2-beat cycle, 137 BPM at 12 fps, 5.26 frames a beat), as meshes near the camera and as impostors further off (the same
poses rendered once to sprites with their normals, so they light the same way). The stadium is built in Blender
(`audience/stadium_build.py`). Everything else (the crowd's places, the beams, the lasers, the light) is here.

Needs numpy, Pillow, ffmpeg, Blender (`/opt/homebrew/bin/blender`) for prep, and the demos' clips
(in `$CARD_ATTACK_WORK/demos/`; the source files are not in this repository).
"""
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))

def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


WORK = _work("audience", "")
BLENDER = os.environ.get("BLENDER", "/opt/homebrew/bin/blender")
W, H, FPS = 1536, 768, 12
F = 1000.0                     # the focal length in pixels at 1536 wide (75 degrees across; the kit takes any)
BPM = 137.14                   # the soundtrack's tempo
BEAT = FPS * 60.0 / BPM        # frames a beat: 5.26 at 12 fps
BAR = 4 * BEAT / FPS           # seconds a bar: 1.75
BEAT_T0 = 0.378                # the beat's phase against the soundtrack's mix: beats 1 and 3 at BEAT_T0 + 0.875 j
CHUNK = 2 * BEAT / FPS         # seconds of each demo in the giant screen's montage: 2 beats, 0.875
NPHASE = 12                    # a move is a 2-beat cycle
T_ARRIVE = 5.0                 # seconds: the camera is down over the crowd
T_END = float(os.environ.get("STADIUM_END", "47.25"))   # the film's end (a hard cut to black on its last frame):
                                                         # the party after the crest stretches to fill it
COUNT = 65536
PARTY_COUNT = 32768             # the party version's crowd (see make_crowd)

MODELS = ["man_a", "man_b", "man_c", "woman_a", "woman_b"]
HEIGHT = {"man_a": 1.80, "man_b": 1.76, "man_c": 1.78, "woman_a": 1.66, "woman_b": 1.68}
MOVES = ["jump", "cheer", "raise", "pump", "sway", "bob", "dance", "point"]


# ======================================================================================================================
# prep: Blender

def prep():
    os.makedirs(WORK, exist_ok=True)
    dl = WORK + "dl"
    subprocess.run([BLENDER, "-b", "--factory-startup", "-P", f"{HERE}/audience/pose_people.py", "--", dl,
                    WORK + "people.npz"], check=True)
    subprocess.run([BLENDER, "-b", "--factory-startup", "-P", f"{HERE}/audience/stadium_build.py", "--",
                    WORK + "stadium"], check=True)


# ======================================================================================================================
# the dancers

class People:
    """The posed meshes in metres, facing +Y (toward the stage), feet at z = 0; their smooth normals; the hands."""

    def __init__(self):
        z = np.load(WORK + "people.npz")
        self.t, self.v, self.n, self.hands, self.lift = {}, {}, {}, {}, {}
        for m in MODELS:
            t = z[m + "/t"]
            self.t[m] = t
            v0 = z[m + "/bob"][0]
            s = HEIGHT[m] / (v0[:, 2].max() - v0[:, 2].min())
            rot = np.array([[-1, 0, 0], [0, -1, 0], [0, 0, 1.0]]) * s      # turned to face +Y
            for mv in MOVES + ["desk", "seat_cheer"]:
                v = z[f"{m}/{mv}"] @ rot.T
                self.v[m, mv] = v.astype(np.float32)
                self.n[m, mv] = np.stack([vertex_normals(v[p], t) for p in range(NPHASE)]).astype(np.float32)
                self.hands[m, mv] = z[f"{m}/{mv}/hands"] @ rot.T
                self.lift[m, mv] = z[f"{m}/{mv}/lift"]


def vertex_normals(v, t):
    """Smooth normals: corners at the same place share one (the models are cut along their flat faces)."""
    _, weld = np.unique(np.round(v, 4), axis=0, return_inverse=True)
    weld = weld.ravel()
    a, b, c = v[t[:, 0]], v[t[:, 1]], v[t[:, 2]]
    fn = np.cross(b - a, c - a)
    n = np.zeros((weld.max() + 1, 3))
    for k in range(3):
        np.add.at(n, weld[t[:, k]], fn)
    n = n[weld]
    return n / (np.linalg.norm(n, axis=1, keepdims=True) + 1e-12)


# the impostors: every pose rendered once, from behind and a little above, to a sprite with its normals

SPR_W, SPR_H = 56, 96
SPR_MW, SPR_MH = 1.5, 2.6       # the sprite's size in metres (a person 1.8 m with the arms up fits)
SPR_ELEV = math.radians(14)


def make_atlas(people):
    path = WORK + "atlas.npz"
    if os.path.exists(path):
        d = np.load(path)
        return d["atlas"], d["hand"]
    n = len(MODELS) * len(MOVES) * NPHASE
    atlas = np.zeros((n, SPR_H, SPR_W, 4), np.float16)
    hand = np.zeros((n, 2, 2), np.float32)        # the right and left hands in sprite metres (across, up)
    ce, se = math.cos(SPR_ELEV), math.sin(SPR_ELEV)
    for mi, m in enumerate(MODELS):
        for vi, mv in enumerate(MOVES):
            for p in range(NPHASE):
                k = (mi * len(MOVES) + vi) * NPHASE + p
                v, nn, t = people.v[m, mv][p], people.n[m, mv][p], people.t[m]
                # the view from behind (-Y) and above: across = x, up = z ce - y se ... toward the camera = -(y ce + z se)
                up = v[:, 2] * ce + v[:, 1] * se
                depth = v[:, 1] * ce - v[:, 2] * se
                sx = (v[:, 0] / SPR_MW + 0.5) * SPR_W
                sy = (1 - (up + 0.1) / SPR_MH) * SPR_H
                tri, bary, _ = raster_ortho(sx[t], sy[t], depth[t], SPR_W, SPR_H, ss=3)
                cov = tri >= 0
                nrm = (nn[t[tri[cov]]] * bary[cov][:, :, None]).sum(1)
                # into the sprite's frame: across, up, toward the camera
                ns = np.stack([nrm[:, 0], nrm[:, 2] * ce + nrm[:, 1] * se, -(nrm[:, 1] * ce - nrm[:, 2] * se)], 1)
                ns /= np.linalg.norm(ns, axis=1, keepdims=True) + 1e-9
                img = np.zeros((SPR_H * 3, SPR_W * 3, 4))
                img.reshape(-1, 4)[cov.ravel()] = np.c_[np.ones(cov.sum()), ns]
                img = img.reshape(SPR_H, 3, SPR_W, 3, 4).mean((1, 3))
                a = img[..., 0:1]
                img[..., 1:] = np.where(a > 0, img[..., 1:] / np.maximum(a, 1e-6), 0)
                atlas[k] = img
                for s_, h in enumerate(people.hands[m, mv][p]):
                    hand[k, s_] = (h[0], h[2] * ce + h[1] * se + people.lift[m, mv][p])
    np.savez_compressed(path, atlas=atlas, hand=hand)
    return atlas, hand


def raster_ortho(sx, sy, depth, w, h, ss=1):
    """A small rasteriser for the sprites (supersampled ss x ss): the nearest triangle per sample and its weights."""
    sx, sy = sx * ss, sy * ss
    w, h = w * ss, h * ss
    xy = np.stack([sx, sy], -1)
    frag = raster(xy, depth, w, h)
    if frag is None:
        return np.full(w * h, -1), np.zeros((w * h, 3)), None
    p, z, t, b = frag
    order = np.lexsort((z, p))
    p, t, b = p[order], t[order], b[order]
    first = np.r_[True, p[1:] != p[:-1]]
    tri = np.full(w * h, -1)
    bary = np.zeros((w * h, 3))
    tri[p[first]] = t[first]
    bary[p[first]] = b[first]
    return tri.reshape(h, w), bary.reshape(h, w, 3), None


# ======================================================================================================================
# the rasteriser: every triangle's pixels as fragments; the nearest kept (the card sorter's way, as the mesh kit does)

def raster(xy, iz_or_z, w, h, persp=False):
    """xy (m, 3, 2) pixel coordinates (pixel centres at i + 0.5); z (m, 3) depths, or 1/z if persp. Returns the
    fragments: pixel index, depth, triangle, the weights (m, 3) (perspective-correct if persp)."""
    x, y = xy[..., 0], xy[..., 1]
    px0 = np.maximum(np.ceil(x.min(1) - 0.5), 0).astype(np.int64)
    px1 = np.minimum(np.floor(x.max(1) - 0.5), w - 1).astype(np.int64)
    py0 = np.maximum(np.ceil(y.min(1) - 0.5), 0).astype(np.int64)
    py1 = np.minimum(np.floor(y.max(1) - 0.5), h - 1).astype(np.int64)
    area = (x[:, 1] - x[:, 0]) * (y[:, 2] - y[:, 0]) - (y[:, 1] - y[:, 0]) * (x[:, 2] - x[:, 0])
    ok = (px1 >= px0) & (py1 >= py0) & (np.abs(area) > 1e-9)
    bw = np.where(ok, px1 - px0 + 1, 0)
    bh = np.where(ok, py1 - py0 + 1, 0)
    cw = np.ceil(np.log2(np.maximum(bw, 1))).astype(int)
    ch = np.ceil(np.log2(np.maximum(bh, 1))).astype(int)
    sgn = np.sign(area)
    outs = []
    for kw in np.unique(cw[ok]):
        for kh in np.unique(ch[ok & (cw == kw)]):
            sel = np.nonzero(ok & (cw == kw) & (ch == kh))[0]
            gw, gh = 1 << kw, 1 << kh
            step = max(1, 4_000_000 // (gw * gh))
            ox, oy = np.arange(gw), np.arange(gh)
            for s in range(0, len(sel), step):
                ids = sel[s:s + step]
                X = px0[ids, None, None] + ox[None, None, :]
                Y = py0[ids, None, None] + oy[None, :, None]
                cx, cy = X + 0.5, Y + 0.5
                xs, ys = x[ids][:, :, None, None], y[ids][:, :, None, None]
                sg = sgn[ids, None, None]
                w0 = ((xs[:, 2] - xs[:, 1]) * (cy - ys[:, 1]) - (ys[:, 2] - ys[:, 1]) * (cx - xs[:, 1])) * sg
                w1 = ((xs[:, 0] - xs[:, 2]) * (cy - ys[:, 2]) - (ys[:, 0] - ys[:, 2]) * (cx - xs[:, 2])) * sg
                w2 = ((xs[:, 1] - xs[:, 0]) * (cy - ys[:, 0]) - (ys[:, 1] - ys[:, 0]) * (cx - xs[:, 0])) * sg
                ins = (w0 >= 0) & (w1 >= 0) & (w2 >= 0) & (X <= px1[ids, None, None]) & (Y <= py1[ids, None, None])
                if not ins.any():
                    continue
                tot = np.abs(area[ids])[:, None, None]
                T = np.broadcast_to(ids[:, None, None], ins.shape)[ins]
                b0, b1 = (w0 / tot)[ins], (w1 / tot)[ins]
                b = np.stack([b0, b1, 1 - b0 - b1], 1)
                P = (Y * w + X)
                P = np.broadcast_to(P, ins.shape)[ins]
                outs.append((P, T, b))
    if not outs:
        return None
    P = np.concatenate([o[0] for o in outs])
    T = np.concatenate([o[1] for o in outs])
    b = np.concatenate([o[2] for o in outs])
    zz = iz_or_z[T]
    if persp:
        q = b * zz
        s = q.sum(1)
        b = q / s[:, None]
        z = 1.0 / s
    else:
        z = (b * zz).sum(1)
    return P, z, T, b


def clip_near(pc, attrs, near):
    """Triangles in the camera's space (m, 3, 3: x right, y up, z forward) cut at z = near. attrs (m, 3, a) are
    carried. Returns the new triangles, their attributes and the index of the triangle each came from."""
    behind = pc[:, :, 2] < near
    nb = behind.sum(1)
    keep = np.nonzero(nb == 0)[0]
    out_p, out_a, out_i = [pc[keep]], [attrs[keep]], [keep]
    for case in (1, 2):
        ids = np.nonzero(nb == case)[0]
        if len(ids) == 0:
            continue
        b = behind[ids]
        # rotate so the odd corner (the one behind for case 1, the one in front for case 2) comes first
        odd = np.argmax(b if case == 1 else ~b, axis=1)
        order = (odd[:, None] + np.arange(3)[None, :]) % 3
        P = np.take_along_axis(pc[ids], order[:, :, None], 1)
        A = np.take_along_axis(attrs[ids], order[:, :, None], 1)

        def cut(i, j):
            t = (near - P[:, i, 2]) / (P[:, j, 2] - P[:, i, 2])
            return (P[:, i] + t[:, None] * (P[:, j] - P[:, i]), A[:, i] + t[:, None] * (A[:, j] - A[:, i]))
        p01, a01 = cut(0, 1)
        p02, a02 = cut(0, 2)
        if case == 1:    # corner 0 behind: a quad 01, 1, 2, 02
            out_p += [np.stack([p01, P[:, 1], P[:, 2]], 1), np.stack([p01, P[:, 2], p02], 1)]
            out_a += [np.stack([a01, A[:, 1], A[:, 2]], 1), np.stack([a01, A[:, 2], a02], 1)]
            out_i += [ids, ids]
        else:            # only corner 0 in front
            out_p.append(np.stack([P[:, 0], p01, p02], 1))
            out_a.append(np.stack([A[:, 0], a01, a02], 1))
            out_i.append(ids)
    return np.concatenate(out_p), np.concatenate(out_a), np.concatenate(out_i)


# ======================================================================================================================
# the scene

def srgb_lin(c):
    return np.power(np.clip(c, 0, 1), 2.2)


class Scene:
    def __init__(self):
        import json
        d = np.load(WORK + "stadium/stadium.npz")
        self.info = json.load(open(WORK + "stadium/stadium.json"))
        self.sp, self.sn, self.sm = d["p"].astype(np.float32), d["n"].astype(np.float32), d["m"]
        mats = self.info["materials"]
        self.m_rgb = np.array([m["rgb"] for m in mats], np.float32)
        self.m_emit = np.array([m.get("emit", 0.0) if m["kind"] == "emit" else 0.0 for m in mats], np.float32)
        self.m_facade = [m["name"] for m in mats].index("facade")
        self.people = People()
        self.atlas, self.atlas_hand = make_atlas(self.people)
        self.screen = self.info["screen"]
        self.lights = self.info["lights"]
        self.spots = [l for l in self.lights if l["type"] == "spot"]
        self.search = [l for l in self.lights if l["type"] == "search"]
        if BLIMP_PLACE is not None:      # the uplights the night arrival placed, on the hull
            for j, u in enumerate(BLIMP_PLACE["uplights"]):
                if "sweep" in u:       # the spots sweeping over the hull
                    self.search.append(dict(type="search", pos=u["source"], rgb=[1, 1, 0.95], power=1.0,
                                            phase=u["phase"], aim=True, sweep=True))
                else:
                    self.search.append(dict(type="search", pos=u["source"], rgb=[1, 1, 0.95], power=1.0,
                                            phase=j * 0.7, aim=True, hull=u["target_on_hull"]))
        else:
            for j, l in enumerate([self.search[i] for i in (1, 6, 8, 17)]):    # four of them up at the airship
                l["aim"] = [(-35, 35, -10, 10)[j], (0, 6, -6, 0)[j]]
        self.lasers = [l for l in self.lights if l["type"] == "lasers"]
        self.make_crowd()
        self.make_city()
        self.make_confetti()
        self.make_callouts()
        self.make_balloons()
        self.load_blimp()
        self.demos = load_demos(self.callouts)
        rng = np.random.default_rng(5150)
        groups = sorted({g for _, g in self.callouts})
        self.shirt_groups = groups
        self.shirt_img = [shirt_image(g, SHIRT_INK[i % len(SHIRT_INK)], logo_for(self.demos, g))
                          for i, g in enumerate(groups)]
        self.cshirt = np.where(rng.random(len(self.cx)) < 0.55, rng.integers(0, len(groups), len(self.cx)), -1)

    # --- the crowd: 65,536 people, on the pitch and on the tiers -----------------------------------------------------
    def make_crowd(self):
        rng = np.random.default_rng(1993)
        b = self.info["bowl"]
        a = self.info["areas"][0]
        sp = 0.55
        xs = np.arange(a["x0"], a["x1"], sp)
        ys = np.arange(a["y0"], a["y1"], sp * 0.95)
        X, Y = np.meshgrid(xs, ys)
        X = X + rng.uniform(-0.2, 0.2, X.shape) + (np.arange(len(ys))[:, None] % 2) * sp / 2
        Y = Y + rng.uniform(-0.15, 0.15, Y.shape)
        X, Y = X.ravel(), Y.ravel()
        # inside the oval of the tiers' inner edge, clear of a little aisle under the camera's path
        inside = (X / (b["A0"] - 1.5)) ** 2 + (Y / (b["B0"] - 1.5)) ** 2 < 1
        path = np.array([camera(t)[0] for t in np.arange(T_ARRIVE - 1.0, T_END, 0.05)])
        low = path[path[:, 2] < 6.0]
        for q in low:
            inside &= (X - q[0]) ** 2 + (Y - q[1]) ** 2 > 0.9 ** 2
        X, Y = X[inside], Y[inside]
        Z = np.zeros_like(X)
        face = np.full_like(X, math.pi / 2) + rng.normal(0, 0.18, X.shape)    # toward +Y, the stage
        kind = np.zeros(len(X), int)
        # the tiers: a row of seats on each step, standing
        tx, ty, tz, tf = [], [], [], []
        for k in range(b["rows"]):
            rx, ry = b["A0"] + (k + 0.5) * b["run"], b["B0"] + (k + 0.5) * b["run"]
            per = 2 * math.pi * math.sqrt((rx * rx + ry * ry) / 2)
            n = int(per / 0.455)      # (0.47 until the weave's aisle through the crowd: 65,536 still)
            am = (np.arange(n) + rng.uniform(-0.2, 0.2, n) + (k % 2) * 0.5) * 2 * math.pi / n
            x, y = rx * np.cos(am), ry * np.sin(am)
            okk = y < b["open_y"] - 2
            tx.append(x[okk]); ty.append(y[okk])
            tz.append(np.full(okk.sum(), b["z0"] + k * b["rise"]))
            # they face the stage end of the pitch
            tf.append(np.arctan2(30.0 - y[okk], 0.0 - x[okk]) + rng.normal(0, 0.15, okk.sum()))
        tx, ty, tz, tf = map(np.concatenate, (tx, ty, tz, tf))
        X = np.r_[X, tx]; Y = np.r_[Y, ty]; Z = np.r_[Z, tz]; face = np.r_[face, tf]
        kind = np.r_[kind, np.ones(len(tx), int)]
        n = len(X)
        if n > COUNT:          # exactly 65,536: drop a random few (more from the tiers' far reaches)
            keep = np.sort(rng.choice(n, COUNT, replace=False))
            X, Y, Z, face, kind = X[keep], Y[keep], Z[keep], face[keep], kind[keep]
        print(f"crowd: {len(X)} ({(kind == 0).sum()} on the pitch, {(kind == 1).sum()} on the tiers)"
              + ("" if len(X) == COUNT else f"; short of {COUNT}"), file=sys.stderr)
        n = len(X)
        self.cx, self.cy, self.cz, self.cface, self.ckind = X, Y, Z, face, kind
        self.cmodel = rng.integers(0, len(MODELS), n)
        self.cscale = rng.uniform(0.93, 1.06, n)
        self.cphase = rng.integers(-1, 2, n)               # in step with the beat, give or take a frame
        # their moves: on the pitch mostly jumping, arms up, phones up; the tiers bob and raise their phones
        r = rng.random(n)
        pitch_moves = np.select([r < 0.30, r < 0.50, r < 0.72, r < 0.84, r < 0.92], [0, 1, 2, 3, 4], 6)
        tier_moves = np.select([r < 0.45, r < 0.75, r < 0.88], [5, 2, 4], 1)
        self.cmove = np.where(kind == 0, pitch_moves, tier_moves)
        self.ctone = rng.uniform(0.6, 1.4, n)              # the clothes' darkness
        # the party version's crowd (32,768, for the live film's budget): everyone within 12 m of
        # the camera's low flight kept (the near rings), the rest thinned, the tiers more than the pitch (far off,
        # the crowd reads as texture)
        path = np.array([camera(t)[0] for t in np.arange(T_ARRIVE - 1.0, T_END, 0.1)])
        low = path[path[:, 2] < 6.0]
        dmin = np.full(n, 1e9)
        for q in low:
            dmin = np.minimum(dmin, np.hypot(X - q[0], Y - q[1]))
        near = dmin < 12.0
        pri = np.random.default_rng(32768).random(n) * np.where(kind == 1, 1.6, 1.0)
        pri[near] = -1.0
        self.cparty = np.zeros(n, bool)
        self.cparty[np.argsort(pri, kind="stable")[:PARTY_COUNT]] = True
        print(f"party crowd: {self.cparty.sum()} ({near.sum()} near the flight; pitch {(self.cparty & (kind == 0)).sum()}, "
              f"tiers {(self.cparty & (kind == 1)).sum()})", file=sys.stderr)

    def moves_at(self, f):
        """Each person's move and phase at frame f: the waves. On the pitch a band of arms-up rolls from the stage
        to the back every 4 bars; on the tiers a Mexican wave goes round the bowl."""
        t = f / FPS
        mv = self.cmove.copy()
        bar = BAR                                            # a bar: 4 beats
        wave_y = 40 - ((t % (4 * bar)) / (4 * bar)) * 140   # from the stage back
        pitch = self.ckind == 0
        band = pitch & (np.abs(self.cy - wave_y) < 7)
        mv[band] = np.where(self.cmove[band] == 2, 2, 1)   # arms up (the phones stay up)
        ang = np.arctan2(self.cy / 58.0, self.cx / 78.0)
        wave_a = -math.pi / 2 + 1.1 * t
        tier = (self.ckind == 1) & (np.abs(np.angle(np.exp(1j * (ang - wave_a)))) < 0.22)
        mv[tier] = 0                                         # the Mexican wave: up, arms up, jumping
        phase = (int(math.floor(f * (NPHASE / 2) / BEAT)) + self.cphase) % NPHASE   # two beats a cycle
        return mv, phase

    # --- the city around, the sky ---------------------------------------------------------------------------------
    def make_city(self):
        rng = np.random.default_rng(7)
        pts, cols = [], []
        # street lights along a grid of roads, out to 1.4 km, none inside the stadium
        for k in range(-14, 15):
            for along in (True, False):
                n = 90
                s = np.linspace(-1400, 1400, n) + rng.uniform(-6, 6, n)
                c = k * 100.0 + 35
                p = np.stack([s, np.full(n, c), np.full(n, 6.0)], 1) if along else np.stack([np.full(n, c), s,
                                                                                              np.full(n, 6.0)], 1)
                pts.append(p)
                cols.append(np.tile([[1.0, 0.55, 0.2]], (n, 1)))
        # windows and lots: scattered warm and white points
        n = 2500
        r = rng.uniform(260, 1400, n)
        a = rng.uniform(0, 2 * math.pi, n)
        pts.append(np.stack([r * np.cos(a), r * np.sin(a), rng.uniform(1, 25, n)], 1))
        cols.append(np.where(rng.random((n, 1)) < 0.5, [[1.0, 0.8, 0.5]], [[0.8, 0.9, 1.0]]) * rng.uniform(0.3, 1, (n, 1)))
        # the car parks round the stadium: rows of lamps
        for ring in (190, 230):
            n = 120
            a = np.linspace(0, 2 * math.pi, n, endpoint=False)
            pts.append(np.stack([ring * 1.15 * np.cos(a), ring * np.sin(a), np.full(n, 10.0)], 1))
            cols.append(np.tile([[1.0, 0.95, 0.85]], (n, 1)))
        p = np.concatenate(pts)
        c = np.concatenate(cols)
        out = (p[:, 0] / 150) ** 2 + (p[:, 1] / 130) ** 2 > 1
        self.city, self.city_rgb = p[out].astype(np.float32), (c[out] * 0.9).astype(np.float32)
        nst = 2500
        d = rng.normal(size=(nst, 3))
        d[:, 2] = np.abs(d[:, 2]) + 0.05
        self.stars = d / np.linalg.norm(d, axis=1, keepdims=True)
        self.star_b = rng.uniform(0.15, 0.6, nst) ** 2

    def make_callouts(self):
        """The demos called out: flags flown in the crowd and LED banners round the bowl, each a title and its
        group as plain text (after the licence audit, the parodies' names for the 55 we may not show)."""
        self.callouts = load_callouts()
        n = len(self.callouts)
        self.callout_img = [callout_image(t_, g_, PALETTE[i % len(PALETTE)]) for i, (t_, g_) in
                            enumerate(self.callouts)]
        rng = np.random.default_rng(64)
        pitch = np.nonzero(self.ckind == 0)[0]
        path = np.array([camera(t)[0] for t in np.arange(T_ARRIVE - 0.5, T_END, 0.1)])
        low = path[path[:, 2] < 6.0]
        dmin = np.full(len(pitch), 1e9)
        for q in low:
            dmin = np.minimum(dmin, np.hypot(self.cx[pitch] - q[0], self.cy[pitch] - q[1]))
        near = pitch[(dmin > 2.5) & (dmin < 9.0)]
        far = pitch[dmin >= 9.0]
        pick = np.r_[rng.choice(near, 30, replace=False), rng.choice(far, 50, replace=False)]
        self.flag_pid = pick
        # about half the flags are national: the Nordic countries most, then the rest of the scene's (`NATIONS`)
        self.flag_img = list(self.callout_img) + [national_flag(c) for c, _ in NATIONS]
        wts = np.array([w_ for _, w_ in NATIONS], float)
        nat = rng.random(len(pick)) < 0.55
        self.flag_tex = np.where(nat, n + rng.choice(len(NATIONS), len(pick), p=wts / wts.sum()),
                                 rng.permutation(len(pick)) % n)
        self.flag_ph = rng.uniform(0, 2 * math.pi, len(pick))
        # the banners: hung from the canopy's inner edge round the stands, facing the pitch
        b = self.info["bowl"]
        rx, ry = b["A0"] + b["canopy_from"] * b["run"] - 1.0, b["B0"] + b["canopy_from"] * b["run"] - 1.0
        self.banners = []
        for j, am in enumerate(np.linspace(-math.pi + 0.35, -0.35, 9).tolist() + [math.pi - 0.5, math.pi - 0.95, 0.5,
                                                                                    0.95]):
            x, y = rx * math.cos(am), ry * math.sin(am)
            if y > b["open_y"] - 4 or (y < -10 and abs(x) < 85):  # none where the canopy is cut for the crest
                continue
            nrm = np.array([-x, -y, 0.0])
            nrm /= np.linalg.norm(nrm)
            self.banners.append((np.array([x, y, 31.5]), nrm, (j * 7 + 3) % n))

    def make_balloons(self):
        """Balloons let go from the crowd, drifting up into the beams."""
        rng = np.random.default_rng(99)
        n = 600
        self.bal_x = rng.uniform(-60, 60, n)
        self.bal_y = rng.uniform(-50, 35, n)
        near = np.arange(n) < 150                      # a quarter of them near the camera's low path
        q = np.array([camera(t)[0] for t in rng.uniform(5.0, 13.0, near.sum())])
        self.bal_x[near] = q[:, 0] + rng.normal(0, 6, near.sum())
        self.bal_y[near] = q[:, 1] + rng.uniform(4, 30, near.sum())
        self.bal_t0 = rng.uniform(-25, 15, n)
        self.bal_v = rng.uniform(0.9, 1.7, n)
        self.bal_r = rng.uniform(0.32, 0.45, n)
        self.bal_ph = rng.uniform(0, 2 * math.pi, n)
        self.bal_c = np.array(PALETTE_BALLOON)[rng.integers(0, len(PALETTE_BALLOON), n)]

    def load_blimp(self):
        """The advertising airship (the night arrival's `blimp()`, models/blimp.obj: the film's
        axes, y up, its screen toward -z), turned into this frame (X = x, Y = z, Z = y). Its materials: hull,
        greeble, mast, bscreen (the picture), bframe (glowing red-orange)."""
        self.blimp = None
        if not os.path.exists(BLIMP_OBJ):
            print("no airship model", file=sys.stderr)
            return
        V, tris, mats, cur = [], [], [], 0
        names = ["hull", "greeble", "mast", "bscreen", "bframe"]
        for line in open(BLIMP_OBJ):
            if line.startswith("v "):
                x, y, z = map(float, line.split()[1:4])
                V.append((x * BLIMP_SCALE, z * BLIMP_SCALE, y * BLIMP_SCALE))
            elif line.startswith("usemtl"):
                nm = line.split()[1]
                cur = names.index(nm) if nm in names else 0
            elif line.startswith("f "):
                ids = [int(p.split("/")[0]) - 1 for p in line.split()[1:]]
                for k in range(1, len(ids) - 1):
                    tris.append((ids[0], ids[k], ids[k + 1]))
                    mats.append(cur)
        V = np.array(V)
        P = V[np.array(tris)]
        fn = np.cross(P[:, 1] - P[:, 0], P[:, 2] - P[:, 0])
        fn /= np.linalg.norm(fn, axis=1, keepdims=True) + 1e-12
        rng = np.random.default_rng(3)
        hull = P[np.array(mats) == 0].reshape(-1, 3)
        lights = hull[rng.choice(len(hull), 420, replace=False)]
        if BLIMP_PLACE is not None:
            rl = BLIMP_PLACE["running_lights"]
            lights = np.array([l_["p"] for l_ in rl if l_["kind"] != "beacon"])
            bea = np.array([l_["p"] for l_ in rl if l_["kind"] == "beacon"])
        self.blimp = dict(P=P.astype(np.float32), N=np.repeat(fn[:, None], 3, 1).astype(np.float32),
                          M=np.array(mats), lights=lights,
                          beacons=V[V[:, 2] > V[:, 2].max() - 1.0][:6] if (V[:, 2] > V[:, 2].max() - 1.0).any()
                          else V[:2])
        if BLIMP_PLACE is not None and len(bea):
            self.blimp["beacons"] = bea
        print(f"airship: {len(P)} triangles", file=sys.stderr)

    def balloons_at(self, t):
        a = t - self.bal_t0
        z = 2.3 + self.bal_v * a
        on = (a > 0) & (z < 75)
        x = self.bal_x + 1.2 * np.sin(0.6 * t + self.bal_ph) + 0.15 * a
        y = self.bal_y + 0.8 * np.cos(0.5 * t + self.bal_ph)
        return np.nonzero(on)[0], np.stack([x, y, z], 1)

    def make_confetti(self):
        rng = np.random.default_rng(11)
        n = 9000
        self.conf_p = np.stack([rng.uniform(-70, 70, n), rng.uniform(-50, 38, n), rng.uniform(0, 1, n)], 1)
        self.conf_c = np.array([[1, 0.2, 0.3], [1, 0.85, 0.1], [0.2, 0.6, 1], [0.3, 1, 0.4], [1, 1, 1],
                                [1, 0.4, 0.9]])[rng.integers(0, 6, n)]
        self.conf_ph = rng.uniform(0, 2 * math.pi, n)
        self.conf_fall = rng.uniform(1.2, 2.2, n)


# ======================================================================================================================
# the show: the beams' and lasers' directions, the pyro, all on the beat

def beat_pulse(t, decay=4.0):
    """1 on beats 1 and 3 (every other beat), decaying."""
    fb = (t * FPS) / (2 * BEAT)
    return np.exp(-decay * (fb - np.floor(fb)))


def spot_dir(l, t):
    """A moving head's beam: sweeping over the crowd in figures, changing pattern every 2 bars."""
    ph = l["phase"]
    bar = int(t / BAR)
    pat = bar % 3
    w = 2 * math.pi / 4.0
    if pat == 0:      # fans swinging left and right together
        pan = 0.55 * math.sin(w * t + ph * 0.3)
        tilt = 0.55 + 0.15 * math.sin(w * 2 * t + ph)
    elif pat == 1:    # crossing: alternate heads swing opposite ways
        pan = 0.6 * math.sin(w * t) * (1 if int(ph / 0.45) % 2 else -1)
        tilt = 0.5 + 0.2 * math.cos(w * t + ph)
    else:             # circles
        pan = 0.4 * math.sin(w * t + ph * 2)
        tilt = 0.5 + 0.25 * math.cos(w * t + ph * 2)
    # tilt: angle from straight down toward the crowd (-Y)
    d = np.array([math.sin(pan) * math.sin(tilt), -math.cos(pan) * math.sin(tilt), -math.cos(tilt)])
    return d / np.linalg.norm(d)


BLIMP_OBJ = _work("models", "blimp.obj")
BLIMP_JSON = _work("models", "blimp.json")   # its placement (the night arrival's)
BLIMP_SCALE = 1.0
BLIMP_AT = np.array([-15.0, 25.0, 118.0])     # high over the bowl (this frame, metres); blimp.json overrides
BLIMP_DRIFT = np.array([0.4, -0.9, 0.0])      # metres a second
BLIMP_SCREEN = dict(c=np.array([4.0, -24.5, 2.0]), w=50.0, h=28.0, tilt=0.26)
BLIMP_PLACE = None
if os.path.exists(BLIMP_JSON):
    import json as _json
    BLIMP_PLACE = _json.load(open(BLIMP_JSON))
    BLIMP_SCALE = BLIMP_PLACE["scale"]
    BLIMP_AT = np.array(BLIMP_PLACE["centre_at_t0"], float)
    BLIMP_DRIFT = np.array([BLIMP_PLACE["drift_x_per_s"], 0.0, 0.0])
    _s = BLIMP_PLACE["screen"]
    BLIMP_SCREEN = dict(c=np.array(_s["centre"], float), w=_s["width"], h=_s["height"], tilt=_s["tilt_down_rad"])


SWEEP_T0 = 546 / 12.0 - 3.0    # the night arrival's clock at our t = 0 (its crest, extension frame 546, is our t = 3)


def blimp_hull(u, v):
    """The hull's surface as the night arrival's model builds it (train_night.py): u along -1..1, v round."""
    L, RW, RH = 55.0, 21.0, 15.0
    base = max(0.0, 1 - abs(u) ** 2.2) ** 0.55
    side, up = math.cos(v), math.sin(v)
    lobe = (0.5 + 0.5 * math.cos(u * math.pi * 6.5)) ** 2 * max(0.0, up + 0.35) * abs(side) ** 0.5
    r = 1 + 0.16 * lobe - 0.10 * max(0.0, -up) ** 2
    return np.array([L * u, RH * base * r * up * (1.15 if up > 0 else 0.85), RW * base * r * side])


def blimp_at(t):
    return BLIMP_AT + BLIMP_DRIFT * t + np.array([0.0, 0.0, 1.2 * math.sin(t * 0.6)])


def search_dir(l, t, sc=None):
    if l.get("sweep") and sc is not None and sc.blimp is not None:   # sweeping along the hull and up its flanks
        tn = t + SWEEP_T0                                  # the night arrival's clock (its extension's frame / 12)
        u = 0.85 * math.sin(0.55 * tn + l["phase"])
        v = -math.pi / 2 + 1.25 * math.sin(0.4 * tn + l["phase"] * 1.7)
        hx, hy, hz = blimp_hull(u, v) * 1.02             # the model's frame: x along, y up, z across
        tgt = blimp_at(t) + np.array([hx, hz, hy]) * BLIMP_SCALE
        d = tgt - np.array(l["pos"])
        return d / np.linalg.norm(d)
    if l.get("hull") is not None and sc is not None and sc.blimp is not None:   # an uplight on its hull
        tgt = blimp_at(t) + np.array(l["hull"]) + np.array([4 * math.sin(0.5 * t + l["phase"]), 0.0, 0.0])
        d = tgt - np.array(l["pos"])
        return d / np.linalg.norm(d)
    if l.get("aim") and sc is not None and sc.blimp is not None:      # caught on the airship's hull, wandering
        tgt = blimp_at(t) + np.array([l["aim"][0] + 12 * math.sin(0.4 * t + l["phase"]),
                                      l["aim"][1] + 6 * math.cos(0.5 * t + l["phase"]), -14.0])
        d = tgt - np.array(l["pos"])
        return d / np.linalg.norm(d)
    ph = l["phase"]
    az = ph + 0.35 * t
    el = math.radians(62 + 18 * math.sin(0.6 * t + ph))
    return np.array([math.cos(az) * math.cos(el), math.sin(az) * math.cos(el), math.sin(el)])


def laser_beams(l, t):
    """A fan of n beams from the source: flat over the crowd's heads, up into the sky, or a scan; changing by bar."""
    n = l["n"]
    bar = int(t / BAR)
    pat = (bar + int(l["pos"][0] / 20)) % 4
    if JOIN_T <= t < JOIN_T + 2.0:                 # the crest: the fans, flat over the heads and up into the sky
        pat = int(l["pos"][0] / 20) % 2
    s = np.linspace(-1, 1, n)
    spread = math.radians(l["spread"]) / 2
    beat = (t * FPS / BEAT) % 1.0
    if pat == 0:       # a flat fan just over the heads, sweeping up and down
        az = -math.pi / 2 + s * spread
        el = np.full(n, math.radians(2.0 + 3.0 * (0.5 + 0.5 * math.sin(2 * math.pi * t / BAR))))
    elif pat == 1:     # a fan up into the sky, swinging
        az = -math.pi / 2 + s * spread * 0.5 + 0.4 * math.sin(2 * math.pi * t / (2 * BAR))
        el = np.full(n, math.radians(25 + 10 * math.sin(2 * math.pi * t / BAR)))
    elif pat == 2:     # a tunnel: beams fanned in elevation, rotating
        az = -math.pi / 2 + 0.6 * np.sin(s * math.pi + 2 * math.pi * t / BAR)
        el = math.radians(4) + (s + 1) * math.radians(12)
    else:              # chasing: one beam at a time lit, scanning across, strobing on the beat
        az = -math.pi / 2 + s * spread
        el = np.full(n, math.radians(6.0))
    d = np.stack([np.cos(el) * np.cos(az), np.cos(el) * np.sin(az), np.sin(el)], 1)
    on = np.ones(n)
    if pat == 3:
        k = int(t * FPS) % n
        on = np.exp(-0.5 * ((np.arange(n) - k) % n) ** 1.0)
        on = on * (0.4 + 0.6 * (beat < 0.5))
    return d, on


def show(t):
    """The lights' levels on the beat (137 BPM): dark between the hits. The moving heads hit on 1 and 3, the
    searchlights on 2 and 4, the screen and the LED walls pulse; in the last bar of every 4-bar phrase (from the
    arrival) a white strobe flashes on the eighth notes."""
    fb = t * FPS / BEAT
    hb = fb / 2                                           # the lights hit on 1 and 3: every other beat
    ph = hb - math.floor(hb)
    kick = math.exp(-5 * ph)
    off = math.exp(-5 * ((ph + 0.5) % 1.0))               # the searchlights on 2 and 4
    strobing = (t % (4 * BAR)) >= 3 * BAR and t > real_t(10.4) and not (AIRSHIP_HOLD[0] <= t <= AIRSHIP_HOLD[1]) \
        and not t >= HAG_AT[0]                            # none in the orbit, the holds or the end
    strobe = 1.0 if (strobing and (fb * 2) % 1.0 < 0.34) else 0.0
    # the crest: the bowl revealed with everything firing at once, settling into the beat over 2 s
    crest = math.exp(-1.2 * (t - JOIN_T)) if t >= JOIN_T - 0.01 else 0.0
    calm = 0.4 if t >= HAG_AT[0] - 0.3 else 1.0          # the thanks and the CARDIAC on the screen: beams eased off
    return dict(kick=kick, off=off, strobe=strobe, crest=crest,
                spots=(0.05 + 0.95 * max(kick, strobe, 0.85 * crest)) * calm,
                search=0.08 + 0.92 * max(off, crest), screen=0.3 + 0.5 * max(kick, 0.8 * crest),
                lasers=(0.12 + 0.88 * max(kick, strobe, crest)) * calm)


def pyro_level(t):
    """The flame columns: a burst at the crest (the reveal) and on the first beat of every second bar after it,
    decaying in 0.75 s."""
    if t < JOIN_T - 0.02:
        return 0.0
    since = (t - JOIN_T) % (2 * BAR)
    return float(np.exp(-since * 4.0)) if since < 1.0 else 0.0


# ======================================================================================================================
# the camera: from outside and high (where the train's ending joins), down over the canopy, over the crowd

JOIN_T = 3.0   # the night arrival's flight hands over here: joined, the film runs from t = 3
# CAM_KEYS and T_END extend freely: add keys after the last (a longer party) and move T_END to the last key's t.
FIXED_KEYS = [   # t, eye, look-at: before the crest (absolute times; standalone only, the night arrival joins at
    (0.0, (-40.0, -150.0, 10.0), (0.0, -100.0, 45.0)),    # JOIN_T)
    (1.5, (-63.5, -125.6, 52.0), (0.0, -40.0, 30.0)),
]
T_FLIGHT = 3.0      # from the crest on, the party's moves, stretched to end at T_END - BLACK_TAIL
FLIGHT = [   # nominal seconds after the crest, eye, look-at; stretched by (T_CUT - T_FLIGHT) / the last time
    (0.0, (-48.0, -110.0, 48.0), (0.0, 40.0, 10.0)),      # the crest: the night arrival's hand-off (MUSIC_FULL,
                                                          # everything firing; then the orbit inside the bowl, CCW:
    (0.9, (-28.0, -88.0, 43.0), (10.0, 40.0, 10.0)),      # 2.2 m a frame north-east and down)
    (1.9, (25.0, -66.0, 40.0), (0.0, 40.0, 10.0)),        # sweeping east over the crowd, the stage ahead
    (2.6, (55.0, -30.0, 40.0), (-30.0, 20.0, 5.0)),       # over the east side, the crowd, the flags, the stage
    (3.9, (39.0, -7.0, 42.0), (-60.0, -20.0, 20.0)),      # round the north, across the sea of people to the stands
    (5.2, (0.0, 5.0, 42.0), (-60.0, -60.0, 20.0)),
    (6.5, (-39.0, -7.0, 42.0), (40.0, -60.0, 30.0)),
    (7.8, (-10.0, -82.0, 36.0), (5.0, -44.0, 116.0)),     # round to the south, up at the airship, its screen
    (9.1, (-16.0, -74.0, 28.0), (10.0, 40.0, 15.0)),      # and the stage again
    (10.4, (-12.0, -58.0, 14.0), (0.0, 60.0, 10.0)),      # the dive down into the crowd
    (11.6, (0.0, -58.0, 3.0), (0.0, 60.0, 12.0)),         # in the crowd
    (13.6, (-12.0, -40.0, 3.0), (-4.0, 60.0, 12.0)),      # through it, past the flags at eye level
    (15.2, (14.0, -30.0, 3.0), (6.0, 60.0, 12.0)),
    (16.4, (6.0, -48.0, 8.0), (3.0, -20.0, 80.0)),        # tilting up from the arms and phones, rising back ...
    (17.6, (0.0, -86.0, 22.0), (5.0, -44.0, 116.0)),      # ... to the airship: THANK YOU SAUL FINGERMAN, held
    (20.4, (1.0, -88.0, 23.0), (6.0, -44.0, 116.0)),      # (no strobe while it holds)
    (21.6, (0.0, -60.0, 12.0), (0.0, 40.0, 20.0)),        # back down
    (23.2, (6.0, -66.0, 4.0), (0.0, 64.0, 30.0)),         # low in the crowd facing the screen: THANK YOU HAGELBARGER
    (26.8, (0.0, -58.0, 6.0), (0.0, 64.0, 31.0)),         # gliding in under the lasers over the heads
    (30.6, (-2.0, -40.0, 14.0), (0.0, 64.0, 32.0)),       # the final rise toward the screen: the CARDIAC, held
]
HOLD_NOMINAL = (17.6, 20.4)     # the airship's hold: THANK YOU SAUL FINGERMAN on its screen
HAG_NOMINAL = (23.0, 27.2)      # the main screen: THANK YOU D. W. HAGELBARGER (type only)
CARDIAC_NOMINAL = 27.2          # the main screen's last picture: a CARDIAC, big, held to the cut
BLACK_TAIL = 3.0                # the film's last seconds are black (a thunk and its reverb tail over it)
T_CUT = T_END - BLACK_TAIL
CHAT_YAW = math.radians(10.0)   # the chat sits in the right ~380 px: the shot's subject a little left of the middle


# The 7 bars cut from the night's airship orbit (147 frames, 12.25 s) come here as more of the
# low, fast flight through the crowd: after the pass by the flags (nominal 15.2) the camera weaves on among the
# dancers at head height, under the lasers, round to the south, before tilting up to the airship. Everything after
# is 12.25 s later; the beat's clock runs on (7 whole bars).
EXTRA = 7 * BAR                 # 12.25 s
EXTRA_AT = 15.2                 # nominal: after this key
WEAVE = [    # seconds after the key at EXTRA_AT, eye, look-at (metres): heads at about 1.7 m
    (1.8, (22.0, -18.0, 2.3), (40.0, 40.0, 6.0)),         # swinging right, the stage's lasers ahead
    (3.6, (30.0, -3.0, 2.1), (-10.0, 30.0, 8.0)),         # between the dancers, turning north-west
    (5.4, (17.0, 9.0, 2.2), (-40.0, 12.0, 7.0)),          # across the front of the crowd, under the fans
    (7.2, (0.0, 13.0, 2.0), (-50.0, -18.0, 7.0)),         # sweeping west along the front
    (9.0, (-19.0, 4.0, 2.2), (-20.0, -60.0, 10.0)),       # round to the south, the crowd streaming past
    (10.8, (-15.0, -15.0, 2.6), (10.0, -60.0, 18.0)),     # out toward the south stands, rising a little
]


def _scale():
    return (T_END - EXTRA - BLACK_TAIL - T_FLIGHT) / FLIGHT[-1][0]


def real_t(tn):
    """A nominal time after the crest on the show's clock: stretched to the cut, the weave's 12.25 s after EXTRA_AT."""
    return T_FLIGHT + _scale() * tn + (EXTRA if tn > EXTRA_AT + 1e-9 else 0.0)


def _keys():
    keys = FIXED_KEYS + [(real_t(t_), e_, g_) for t_, e_, g_ in FLIGHT]
    t0 = real_t(EXTRA_AT)
    keys += [(t0 + dt, e_, g_) for dt, e_, g_ in WEAVE]
    return sorted(keys, key=lambda k_: k_[0]), _scale()


CAM_KEYS, _K = _keys()
AIRSHIP_HOLD = (real_t(HOLD_NOMINAL[0]) - 0.5, real_t(HOLD_NOMINAL[1]) + 0.5)
HAG_AT = (real_t(HAG_NOMINAL[0]), real_t(HAG_NOMINAL[1]))
CARDIAC_AT = real_t(CARDIAC_NOMINAL)


def catmull(ts, ps, t):
    ts = np.asarray(ts)
    ps = np.asarray(ps, float)
    i = int(np.clip(np.searchsorted(ts, t) - 1, 0, len(ts) - 2))
    p0, p1, p2, p3 = ps[max(i - 1, 0)], ps[i], ps[i + 1], ps[min(i + 2, len(ps) - 1)]
    t0, t1, t2, t3 = ts[max(i - 1, 0)], ts[i], ts[i + 1], ts[min(i + 2, len(ts) - 1)]
    u = (t - t1) / (t2 - t1)
    m1 = (p2 - p0) / max(t2 - t0, 1e-6) * (t2 - t1)
    m2 = (p3 - p1) / max(t3 - t1, 1e-6) * (t2 - t1)
    u2, u3 = u * u, u * u * u
    return (2 * u3 - 3 * u2 + 1) * p1 + (u3 - 2 * u2 + u) * m1 + (-2 * u3 + 3 * u2) * p2 + (u3 - u2) * m2


def camera(t):
    """The eye and the point looked at, at t seconds (metres; the stage at +Y, up +Z). The night arrival joins at
    JOIN_T, the crest over the south rim (CAM_KEYS' key at JOIN_T, to be matched to the pose it posts)."""
    ts = [k[0] for k in CAM_KEYS]
    t = min(max(t, 0.0), ts[-1])
    eye = catmull(ts, [k[1] for k in CAM_KEYS], t)
    tgt = catmull(ts, [k[2] for k in CAM_KEYS], t)
    # turn the aim a little right after the sweep down, so the subject sits left of the chat
    a = CHAT_YAW * float(np.clip((t - T_FLIGHT - 2.0) / 1.5, 0, 1))
    d = tgt - eye
    ca, sa = math.cos(-a), math.sin(-a)
    d = np.array([ca * d[0] - sa * d[1], sa * d[0] + ca * d[1], d[2]])
    return eye, eye + d


def basis(eye, tgt):
    fwd = tgt - eye
    fwd = fwd / np.linalg.norm(fwd)
    right = np.cross(fwd, [0, 0, 1.0])
    right /= np.linalg.norm(right)
    up = np.cross(right, fwd)
    return fwd, right, up


# ======================================================================================================================
# a frame

K_SURF, K_EMIT, K_PERSON, K_SPRITE, K_SCREEN, K_LED, K_PHOTO, K_FLAG, K_BANNER, K_BALL, K_BLIMP, K_SKIN = range(12)


def render(sc, eye, tgt, t, w=W, h=H, stats=None, photos=None):
    f_px = F * w / 1536.0
    eye = np.asarray(eye, float)
    fwd, right, up = basis(eye, np.asarray(tgt, float))
    frame = int(round(t * FPS))
    mv, ph = sc.moves_at(frame)

    def to_cam(p):
        d = p - eye
        return np.stack([d @ right, d @ up, d @ fwd], -1)

    # --- geometry: the stadium; the screen and the LED walls as quads; the people near as meshes, the rest sprites
    P = [sc.sp]
    N = [sc.sn]
    kind = [np.where(sc.m_emit[sc.sm] > 0, K_EMIT, np.where(sc.sm == sc.m_facade, K_SKIN, K_SURF))]
    alb = [sc.m_rgb[sc.sm]]
    emi = [sc.m_rgb[sc.sm] * sc.m_emit[sc.sm][:, None]]
    UV = [np.zeros((len(sc.sm), 3, 2), np.float32)]
    aux = [np.zeros(len(sc.sm), np.int64)]

    def quad(c, wid, hei, k, aux_v=0, rgt=(1, 0, 0), upv=(0, 0, 1)):
        c, rgt, upv = np.array(c), np.array(rgt, float), np.array(upv, float)
        a, b_, cc, d = (c - rgt * wid / 2 - upv * hei / 2, c + rgt * wid / 2 - upv * hei / 2,
                        c + rgt * wid / 2 + upv * hei / 2, c - rgt * wid / 2 + upv * hei / 2)
        nrm = np.cross(rgt, upv)
        P.append(np.array([[a, b_, cc], [a, cc, d]], np.float32))
        N.append(np.tile(nrm, (2, 3, 1)).astype(np.float32))
        kind.append(np.full(2, k))
        alb.append(np.zeros((2, 3), np.float32))
        emi.append(np.zeros((2, 3), np.float32))
        UV.append(np.array([[[0, 0], [1, 0], [1, 1]], [[0, 0], [1, 1], [0, 1]]], np.float32))
        aux.append(np.full(2, aux_v))

    s = sc.screen
    quad(s["centre"], s["width"], s["height"], K_SCREEN, rgt=(1, 0, 0), upv=(0, 0, 1))
    for i, lw in enumerate(sc.info["led_walls"]):
        quad([lw["centre"][0], lw["centre"][1] - 0.05, lw["centre"][2]], lw["width"], lw["height"], K_LED, aux_v=i)
    for c_, n_, k_ in sc.banners:                     # the callouts' LED banners round the bowl
        rgt = np.array([-n_[1], n_[0], 0.0])
        quad(c_, 12.0, 7.5, K_BANNER, aux_v=k_, rgt=tuple(rgt), upv=(0, 0, 1))
    # the flags: a pole from the holder's hands, the cloth waving
    pos_f = np.stack([sc.cx[sc.flag_pid], sc.cy[sc.flag_pid], sc.cz[sc.flag_pid]], 1)
    gu, gv = 10, 2
    us, vs = np.linspace(0, 1, gu + 1), np.linspace(0, 1, gv + 1)
    for j in range(len(sc.flag_pid)):
        p0 = pos_f[j]
        if np.linalg.norm(p0 - eye) > 140 or (p0 - eye) @ fwd < -3:
            continue
        wf, hf = 2.0, 1.25
        top = p0 + [0, 0, 3.9]
        ph_f = sc.flag_ph[j]
        U, V = np.meshgrid(us, vs)
        X = top[0] + U * wf
        Y = top[1] + 0.28 * U * np.sin(2 * math.pi * (U * 0.9 - 0.8 * t) + ph_f)
        Z = top[2] - (1 - V) * hf - 0.12 * U * U
        pts = np.stack([X, Y, Z], -1)
        dYdU = 0.28 * (np.sin(2 * math.pi * (U * 0.9 - 0.8 * t) + ph_f) + U * 2 * math.pi * 0.9 *
                       np.cos(2 * math.pi * (U * 0.9 - 0.8 * t) + ph_f))
        nn = np.stack([dYdU / wf, -np.ones_like(U), np.zeros_like(U)], -1)
        nn /= np.linalg.norm(nn, axis=-1, keepdims=True)
        uvg = np.stack([U, V], -1)
        tris_p, tris_n, tris_uv = [], [], []
        for a_ in range(gv):
            for b_ in range(gu):
                c00, c10, c11, c01 = (a_, b_), (a_, b_ + 1), (a_ + 1, b_ + 1), (a_ + 1, b_)
                for tri_ in ((c00, c10, c11), (c00, c11, c01)):
                    tris_p.append([pts[c] for c in tri_])
                    tris_n.append([nn[c] for c in tri_])
                    tris_uv.append([uvg[c] for c in tri_])
        nt = len(tris_p)
        P.append(np.array(tris_p, np.float32))
        N.append(np.array(tris_n, np.float32))
        UV.append(np.array(tris_uv, np.float32))
        kind.append(np.full(nt, K_FLAG))
        alb.append(np.zeros((nt, 3), np.float32))
        emi.append(np.zeros((nt, 3), np.float32))
        aux.append(np.full(nt, sc.flag_tex[j]))
        # the pole
        quad(p0 + [0, 0, 2.6 + 0.65], 0.05, 2.6, K_SURF, rgt=(1, 0, 0), upv=(0, 0, 1))
        alb[-1][:] = 0.3
    # the airship
    if sc.blimp is not None:
        bp = blimp_at(t)
        nb_ = len(sc.blimp["M"])
        P.append(sc.blimp["P"] + bp.astype(np.float32))
        N.append(sc.blimp["N"])
        kind.append(np.full(nb_, K_BLIMP))
        alb.append(np.zeros((nb_, 3), np.float32))
        emi.append(np.zeros((nb_, 3), np.float32))
        UV.append(np.zeros((nb_, 3, 2), np.float32))
        aux.append(sc.blimp["M"])
    # the balloons: a quad each, turned to the camera; cut to a disc and given a sphere's normal when shaded
    b_ids, b_pos = sc.balloons_at(t)
    for j in b_ids:
        c_ = b_pos[j]
        r_ = sc.bal_r[j]
        to = eye - c_
        dist_b = np.linalg.norm(to)
        if dist_b > 220 or to @ fwd > 0:
            continue
        rg_ = np.cross(fwd, [0, 0, 1.0])
        rg_ /= np.linalg.norm(rg_)
        quad(c_, 2 * r_, 2.4 * r_, K_BALL, aux_v=j, rgt=tuple(rg_), upv=tuple(np.cross(rg_, fwd)))

    # the people: distance and whether in view
    pos = np.stack([sc.cx, sc.cy, sc.cz], 1)
    rel = pos + np.array([0, 0, 1.0]) - eye
    dist = np.linalg.norm(rel, axis=1)
    zc = rel @ fwd
    xc, yc = rel @ right, rel @ up
    vis = (zc > 0.3) & (np.abs(xc) < zc * (w / 2 / f_px) + 2.5) & (np.abs(yc) < zc * (h / 2 / f_px) + 2.5)
    near_ids = np.nonzero(vis & (dist < 26.0))[0]
    near_ids = near_ids[np.argsort(dist[near_ids])][:160]
    is_near = np.zeros(len(pos), bool)
    is_near[near_ids] = True
    lift = np.zeros(len(pos))
    photo_ids = near_ids if photos is not None else np.zeros(0, int)
    if photos is not None:
        near_ids = np.zeros(0, int)
        is_near[:] = False
    for i in near_ids:
        m = MODELS[sc.cmodel[i]]
        mvn = MOVES[mv[i]]
        p = ph[i]
        v = sc.people.v[m, mvn][p] * sc.cscale[i]
        nn = sc.people.n[m, mvn][p]
        a = sc.cface[i] - math.pi / 2
        ca, sa = math.cos(a), math.sin(a)
        R = np.array([[ca, -sa, 0], [sa, ca, 0], [0, 0, 1.0]])
        l_ = sc.people.lift[m, mvn][p]
        lift[i] = l_
        vw = v @ R.T + pos[i] + [0, 0, l_]
        tt = sc.people.t[m]
        P.append(vw[tt].astype(np.float32))
        N.append((nn @ R.T)[tt].astype(np.float32))
        kind.append(np.full(len(tt), K_PERSON))
        tone = 0.025 * sc.ctone[i]
        alb.append(np.tile(np.array([tone, tone * 0.95, tone * 1.15], np.float32), (len(tt), 1)))
        emi.append(np.zeros((len(tt), 3), np.float32))
        UV.append(np.zeros((len(tt), 3, 2), np.float32))
        aux.append(np.full(len(tt), i))
    # the sprites: a quad each, upright, turned to the camera
    sid = np.nonzero(vis & ~is_near)[0]
    if len(sid):
        mi, vi, pi = sc.cmodel[sid], mv[sid], ph[sid]
        lifts = np.zeros(len(sid))
        for a_m, m in enumerate(MODELS):
            for a_v, mvn in enumerate(MOVES):
                sel = (mi == a_m) & (vi == a_v)
                if sel.any():
                    lifts[sel] = sc.people.lift[m, mvn][pi[sel]]
        c = pos[sid] + np.c_[np.zeros((len(sid), 2)), lifts - 0.1]
        to = eye[None, :2] - c[:, :2]
        to /= np.linalg.norm(to, axis=1, keepdims=True) + 1e-9
        rg = np.c_[-to[:, 1], to[:, 0], np.zeros(len(sid))]          # right, seen from the camera
        sw = SPR_MW * sc.cscale[sid]
        sh = SPR_MH * sc.cscale[sid]
        upv = np.array([0, 0, 1.0])
        a = c - rg * sw[:, None] / 2
        b_ = c + rg * sw[:, None] / 2
        cc = b_ + upv * sh[:, None]
        d = a + upv * sh[:, None]
        Pq = np.concatenate([np.stack([a, b_, cc], 1), np.stack([a, cc, d], 1)]).astype(np.float32)
        nrm = np.c_[to, np.zeros(len(sid))]
        P.append(Pq)
        N.append(np.tile(np.concatenate([nrm, nrm])[:, None, :], (1, 3, 1)).astype(np.float32))
        kind.append(np.full(2 * len(sid), K_SPRITE))
        tone = 0.025 * sc.ctone[sid]
        al = np.stack([tone, tone * 0.95, tone * 1.15], 1)
        alb.append(np.concatenate([al, al]).astype(np.float32))
        emi.append(np.zeros((2 * len(sid), 3), np.float32))
        uvq = np.array([[[0, 0], [1, 0], [1, 1]], [[0, 0], [1, 1], [0, 1]]], np.float32)
        UV.append(np.concatenate([np.tile(uvq[0], (len(sid), 1, 1)), np.tile(uvq[1], (len(sid), 1, 1))]))
        k_at = (mi * len(MOVES) + vi) * NPHASE + pi
        aux.append(np.concatenate([sid, sid]))
        sprite_k = np.zeros(len(pos), np.int64)
        sprite_k[sid] = k_at
        sprite_rg = np.zeros((len(pos), 3))
        sprite_rg[sid] = rg
        lift[sid] = lifts
    else:
        sprite_k = np.zeros(len(pos), np.int64)
        sprite_rg = np.zeros((len(pos), 3))

    for j, i in enumerate(photo_ids):     # the comparison: a photo cut-out, upright, turned to the camera
        k_ = j % len(photos["imgs"])
        hm, top = photos["size"][k_]
        img = photos["imgs"][k_]
        wm = hm * img.shape[1] / img.shape[0]
        bob = sc.people.lift[MODELS[sc.cmodel[i]], MOVES[mv[i]]][ph[i]]
        c = pos[i] + [0, 0, top * sc.cscale[i] + bob - hm / 2]
        to = eye[:2] - c[:2]
        to = to / (np.linalg.norm(to) + 1e-9)
        rgt = (-to[1], to[0], 0.0)
        quad(c, wm, hm, K_PHOTO, aux_v=k_, rgt=rgt, upv=(0, 0, 1))
        N[-1][:] = np.array([to[0], to[1], 0.0])

    P = np.concatenate(P)
    N = np.concatenate(N)
    kind = np.concatenate(kind)
    alb = np.concatenate(alb)
    emi = np.concatenate(emi)
    UV = np.concatenate(UV)
    aux = np.concatenate(aux)

    # --- into the camera, cut at the near plane, onto the screen
    pc = to_cam(P)
    near = 0.15
    attrs = np.concatenate([P, N, UV], -1)
    infront = (pc[:, :, 2] > near).any(1)
    pc, attrs, src = clip_near(pc[infront], attrs[infront], near)
    src = np.nonzero(infront)[0][src]
    zc_ = pc[:, :, 2]
    sx = w / 2 + f_px * pc[:, :, 0] / zc_
    sy = h / 2 - f_px * pc[:, :, 1] / zc_
    frag = raster(np.stack([sx, sy], -1), 1.0 / zc_, w, h, persp=True)
    if frag is None:
        return sky(sc, eye, fwd, right, up, f_px, w, h, t)
    pix, z, tri, b = frag
    # the sprites' cut-outs
    ks = kind[src[tri]]
    spr = ks == K_SPRITE
    if spr.any():
        uv = (attrs[tri[spr], :, 6:8] * b[spr][:, :, None]).sum(1)
        k_ = sprite_k[aux[src[tri[spr]]]]
        tx = np.clip((uv[:, 0] * SPR_W).astype(int), 0, SPR_W - 1)
        ty = np.clip(((1 - uv[:, 1]) * SPR_H).astype(int), 0, SPR_H - 1)
        alpha = sc.atlas[k_, ty, tx, 0].astype(np.float32)
        keep = np.ones(len(pix), bool)
        keep[np.nonzero(spr)[0][alpha < 0.5]] = False
        pix, z, tri, b = pix[keep], z[keep], tri[keep], b[keep]
    bl = kind[src[tri]] == K_BALL
    if bl.any():
        uvb = (attrs[tri[bl], :, 6:8] * b[bl][:, :, None]).sum(1)
        rr = ((uvb[:, 0] - 0.5) * 2) ** 2 + ((uvb[:, 1] - 0.55) * 2.4) ** 2
        keep = np.ones(len(pix), bool)
        keep[np.nonzero(bl)[0][rr > 1]] = False
        pix, z, tri, b = pix[keep], z[keep], tri[keep], b[keep]
    if photos is not None:
        ks = kind[src[tri]]
        ph_ = ks == K_PHOTO
        if ph_.any():
            uv = (attrs[tri[ph_], :, 6:8] * b[ph_][:, :, None]).sum(1)
            kk = aux[src[tri[ph_]]]
            keep = np.ones(len(pix), bool)
            al = np.zeros(ph_.sum())
            for q, img in enumerate(photos["imgs"]):
                m_ = kk == q
                ty = np.clip(((1 - uv[m_, 1]) * img.shape[0]).astype(int), 0, img.shape[0] - 1)
                tx = np.clip((uv[m_, 0] * img.shape[1]).astype(int), 0, img.shape[1] - 1)
                al[m_] = img[ty, tx, 3]
            keep[np.nonzero(ph_)[0][al < 0.5]] = False
            pix, z, tri, b = pix[keep], z[keep], tri[keep], b[keep]
    order = np.lexsort((z, pix))
    pix, z, tri, b = pix[order], z[order], tri[order], b[order]
    first = np.r_[True, pix[1:] != pix[:-1]]
    pix, z, tri, b = pix[first], z[first], tri[first], b[first]
    if stats is not None:
        stats["triangles"] = len(src)
        stats["pixels"] = len(pix)
        stats["meshes"] = len(near_ids)
        stats["sprites"] = len(sid)

    t_src = src[tri]
    k = kind[t_src]
    A = (attrs[tri] * b[:, :, None]).sum(1)
    wp, nrm, uv = A[:, 0:3], A[:, 3:6], A[:, 6:8]
    nrm /= np.linalg.norm(nrm, axis=1, keepdims=True) + 1e-9
    # sprites: the normal from the atlas, turned into the world
    spr = k == K_SPRITE
    if spr.any():
        pid = aux[t_src[spr]]
        k_ = sprite_k[pid]
        tx = np.clip((uv[spr, 0] * SPR_W).astype(int), 0, SPR_W - 1)
        ty = np.clip(((1 - uv[spr, 1]) * SPR_H).astype(int), 0, SPR_H - 1)
        ns = sc.atlas[k_, ty, tx, 1:4].astype(np.float32)
        rg = sprite_rg[pid]
        toc = np.c_[rg[:, 1], -rg[:, 0], np.zeros(len(pid))]
        nrm[spr] = ns[:, 0:1] * rg + ns[:, 1:2] * np.array([0, 0, 1.0]) + ns[:, 2:3] * toc

    col = np.zeros((len(pix), 3), np.float32)
    view = eye - wp
    vd = view / (np.linalg.norm(view, axis=1, keepdims=True) + 1e-9)

    # the screen's picture and its light
    sh = show(t)
    pic = screen_picture(sc, t) * sh["screen"]
    lw_pics = [led_picture(sc, t, i) for i in range(len(sc.info["led_walls"]))]
    is_scr = k == K_SCREEN
    if is_scr.any():
        u, v = uv[is_scr, 0], uv[is_scr, 1]
        ph_, pw_ = pic.shape[:2]
        col[is_scr] = pic[np.clip(((1 - v) * ph_).astype(int), 0, ph_ - 1), np.clip((u * pw_).astype(int), 0, pw_ - 1)]
    is_led = k == K_LED
    if is_led.any():
        for i, lp in enumerate(lw_pics):
            sel = is_led & (aux[t_src] == i)
            u, v = uv[sel, 0], uv[sel, 1]
            col[sel] = lp[np.clip(((1 - v) * lp.shape[0]).astype(int), 0, lp.shape[0] - 1),
                          np.clip((u * lp.shape[1]).astype(int), 0, lp.shape[1] - 1)]
    is_em = k == K_EMIT
    col[is_em] = emi[t_src[is_em]]
    for kind_, glow in ((K_FLAG, None), (K_BANNER, None)):
        sel = k == kind_
        if not sel.any():
            continue
        tex = np.zeros((sel.sum(), 3), np.float32)
        kk = aux[t_src[sel]]
        for q in np.unique(kk):
            img_ = sc.flag_img[q] if kind_ == K_FLAG else banner_picture(sc, t, q)
            m_ = kk == q
            ty = np.clip(((1 - uv[sel][m_, 1]) * img_.shape[0]).astype(int), 0, img_.shape[0] - 1)
            tx = np.clip((uv[sel][m_, 0] * img_.shape[1]).astype(int), 0, img_.shape[1] - 1)
            tex[m_] = img_[ty, tx]
        if kind_ == K_FLAG:      # cloth: lit by the show, a little light from the stage always on it
            Lf = light_at(sc, t, wp[sel], nrm[sel], vd[sel], pic, lw_pics)
            Lf2 = light_at(sc, t, wp[sel], -nrm[sel], vd[sel], pic, lw_pics)   # light through the cloth
            col[sel] = tex * (Lf + 0.5 * Lf2 + 0.10)
        else:                    # LED banners: their own light, pulsing
            col[sel] = tex * (0.5 + 0.6 * sh["kick"])
    sel = k == K_SKIN
    if sel.any():                 # the bowl's outer wall as an LED skin, as the night arrival has it (callouts scroll)
        col[sel] = skin_light(wp[sel], t) + alb[t_src[sel]] * 0.3 * 0.01
    sel = k == K_BLIMP
    if sel.any():
        mt = aux[t_src[sel]]
        Lb = light_at(sc, t, wp[sel], nrm[sel], vd[sel], pic, lw_pics, airship=True)
        under = np.clip(-nrm[sel][:, 2], 0, 1)[:, None] * np.array([0.10, 0.07, 0.12])   # the bowl's glow from below
        albb = np.array([[0.055, 0.055, 0.065], [0.085, 0.08, 0.085], [0.12, 0.12, 0.13], [0, 0, 0], [0, 0, 0]])[mt]
        cb = albb * (Lb + under * 14.0)
        cb[mt == 4] = np.array([1.0, 0.28, 0.06]) * 1.4                                    # the frame's glow
        scr = mt == 3
        if scr.any():                                                                      # its screen: a demo
            bs = BLIMP_SCREEN
            upv = np.array([0.0, -math.sin(bs["tilt"]), math.cos(bs["tilt"])])
            q = wp[sel][scr] - blimp_at(t) - bs["c"]
            a_ = np.clip(q[:, 0] / bs["w"] + 0.5, 0, 1)
            b_ = np.clip((q @ upv) / bs["h"] + 0.5, 0, 1)
            fr_ = thanks_fingerman(t) if AIRSHIP_HOLD[0] <= t <= AIRSHIP_HOLD[1] else airship_picture(sc, t)
            hh_, ww_ = fr_.shape[:2]
            cb[scr] = fr_[((1 - b_) * (hh_ - 1)).astype(int), (a_ * (ww_ - 1)).astype(int)] * 1.3
        col[sel] = cb
    sel = k == K_BALL
    if sel.any():
        j_ = aux[t_src[sel]]
        u_, v_ = (uv[sel, 0] - 0.5) * 2, (uv[sel, 1] - 0.55) * 2.4
        rg_ = np.cross(fwd, [0, 0, 1.0])
        rg_ /= np.linalg.norm(rg_)
        up_ = np.cross(rg_, fwd)
        nz = np.sqrt(np.clip(1 - u_ * u_ - v_ * v_, 0, 1))
        nb = u_[:, None] * rg_ + v_[:, None] * up_ - nz[:, None] * fwd
        Lb = light_at(sc, t, wp[sel], nb, vd[sel], pic, lw_pics)
        cb = sc.bal_c[j_]
        rim_b = (1 - nz) ** 3
        Lt = light_at(sc, t, wp[sel], -nb, vd[sel], pic, lw_pics)          # the light through the rubber
        spec = np.clip(nz - 0.85, 0, 1) * 4
        col[sel] = cb * (Lb * 0.9 + Lt * 0.7 + 0.06) + Lb * (rim_b + spec)[:, None] * 0.5

    isp = k == K_PHOTO
    if isp.any():
        kk = aux[t_src[isp]]
        rgbp = np.zeros((isp.sum(), 3), np.float32)
        rimp = np.zeros(isp.sum(), np.float32)
        for q, img in enumerate(photos["imgs"]):
            m_ = kk == q
            ty = np.clip(((1 - uv[isp][m_, 1]) * img.shape[0]).astype(int), 0, img.shape[0] - 1)
            tx = np.clip((uv[isp][m_, 0] * img.shape[1]).astype(int), 0, img.shape[1] - 1)
            rgbp[m_] = img[ty, tx, :3]
            rimp[m_] = photos["rims"][q][ty, tx]
        Lp = light_at(sc, t, wp[isp], nrm[isp], vd[isp], pic, lw_pics)
        col[isp] = srgb_lin(rgbp) * 0.08 * Lp.mean(1, keepdims=True) + 0.9 * Lp * rimp[:, None]
    lit = (k == K_SURF) | (k == K_PERSON) | (k == K_SPRITE)
    if lit.any():
        L = light_at(sc, t, wp[lit], nrm[lit], vd[lit], pic, lw_pics)
        a_ = alb[t_src[lit]]
        person = (k[lit] == K_PERSON) | (k[lit] == K_SPRITE)
        # the people: dark, with the rim: light grazing their edges, from the screen and the beams ahead of them
        ndv = np.abs((nrm[lit] * vd[lit]).sum(1))
        rim = np.where(person, 1.2 * (1 - ndv) ** 6, 0.0)
        col[lit] = a_ * L + rim[:, None] * L + a_ * 0.004
        # the near crowd's backs: T-shirts and hoodies printed with the groups' names (UV-reactive ink: it glows a
        # little under the show's blacklight, and the light catches it)
        pm = (k[lit] == K_PERSON)
        if pm.any():
            li = np.nonzero(lit)[0][pm]
            pid = aux[t_src[li]]
            shirt = sc.cshirt[pid]
            has = shirt >= 0
            if has.any():
                li, pid, shirt = li[has], pid[has], shirt[has]
                a = sc.cface[pid] - math.pi / 2
                ca, sa = np.cos(a), np.sin(a)
                d_ = wp[li] - np.stack([sc.cx[pid], sc.cy[pid], sc.cz[pid] + lift[pid]], 1)
                lx = (ca * d_[:, 0] + sa * d_[:, 1]) / sc.cscale[pid]
                ly = (-sa * d_[:, 0] + ca * d_[:, 1]) / sc.cscale[pid]
                lz = d_[:, 2] / sc.cscale[pid]
                u_ = (lx + 0.2) / 0.4
                v_ = (lz - 1.0) / 0.42
                on = (u_ > 0) & (u_ < 1) & (v_ > 0) & (v_ < 1) & (ly < 0.02)
                if on.any():
                    li, shirt, u_, v_ = li[on], shirt[on], u_[on], v_[on]
                    ink = np.zeros((len(li), 4), np.float32)
                    for q in np.unique(shirt):
                        im_ = sc.shirt_img[q]
                        m_ = shirt == q
                        ty = np.clip(((1 - v_[m_]) * im_.shape[0]).astype(int), 0, im_.shape[0] - 1)
                        tx = np.clip((u_[m_] * im_.shape[1]).astype(int), 0, im_.shape[1] - 1)
                        ink[m_] = im_[ty, tx]
                    Lq = light_at(sc, t, wp[li], nrm[li], vd[li], pic, lw_pics)
                    glow = 0.05 + 0.04 * sh["kick"] + 0.25 * sh["strobe"]
                    pr = ink[:, :3] * (Lq * 0.6 + glow)
                    a4 = ink[:, 3:4]
                    col[li] = col[li] * (1 - a4) + pr * a4
    img = sky(sc, eye, fwd, right, up, f_px, w, h, t)
    img = img.reshape(-1, 3)
    img[pix] = col
    depth = np.full(w * h, 1e9, np.float32)
    depth[pix] = z
    img = img.reshape(h, w, 3)
    depth = depth.reshape(h, w)

    # --- the lights that aren't surfaces: city, phones, confetti, the beams through the haze, the lasers, pyro
    points(img, depth, sc.city, sc.city_rgb, eye, fwd, right, up, f_px, size=1.0)
    if sc.blimp is not None:      # its running lights (steady, warm) and red beacons blinking
        bp = blimp_at(t)
        lp = sc.blimp["lights"] + bp
        lp = lp + (lp - bp) * 0.01
        points(img, depth, lp, np.tile(np.array([[1.0, 0.75, 0.4]], np.float32) * 0.9, (len(lp), 1)), eye, fwd,
               right, up, f_px, size=0.7)
        if (t * 1.5) % 1.0 < 0.35:
            bc = sc.blimp["beacons"] + bp
            points(img, depth, bc, np.tile(np.array([[3.0, 0.1, 0.05]], np.float32), (len(bc), 1)), eye, fwd,
                   right, up, f_px, size=1.5)
    phones(sc, img, depth, mv, ph, is_near, sprite_rg, lift, eye, fwd, right, up, f_px, t)
    photo_flashes(sc, img, depth, eye, fwd, right, up, f_px, t)
    if t > JOIN_T - 0.3:
        confetti(sc, img, depth, eye, fwd, right, up, f_px, t)
    img += volumetric(sc, depth, eye, fwd, right, up, f_px, w, h, t)
    if sh["strobe"] > 0:          # the strobe lights the haze: everything nearer stands out black against it
        img += (sh["strobe"] * 0.3 * np.clip((np.minimum(depth, 150.0) - 15.0) / 60.0, 0, 1))[..., None] * np.array([0.9, 0.92, 1.0],
                                                                                          np.float32)
    lasers(sc, img, depth, eye, fwd, right, up, f_px, t)
    pyro(sc, img, depth, eye, fwd, right, up, f_px, t)
    img = bloom(img)
    out = 1 - np.exp(-1.25 * img)
    return np.clip(out, 0, 1) ** (1 / 2.2)


def sky(sc, eye, fwd, right, up, f_px, w, h, t):
    ys, xs = np.mgrid[0:h, 0:w]
    dx = (xs + 0.5 - w / 2) / f_px
    dy = -(ys + 0.5 - h / 2) / f_px
    d = fwd[None, None] + dx[..., None] * right[None, None] + dy[..., None] * up[None, None]
    d /= np.linalg.norm(d, axis=-1, keepdims=True)
    el = d[..., 2]
    hor = np.exp(-np.maximum(el, 0) * 9)
    img = (np.array([0.004, 0.006, 0.016]) * (1 - hor[..., None]) + np.array([0.05, 0.03, 0.035]) * hor[..., None])
    img = img + np.array([0.06, 0.03, 0.01]) * np.exp(-np.maximum(el, 0) * 30)[..., None]      # the city's glow
    img = np.where(el[..., None] < 0, np.array([0.004, 0.004, 0.005]), img).astype(np.float32)
    # the night arrival's sky deepens to dark violet before the hand-off: from JOIN_T on, violet
    fade = float(np.clip(1 - t / JOIN_T, 0, 1))
    vio = np.array([0.012, 0.004, 0.026]) * (1 - hor[..., None]) + np.array([0.050, 0.018, 0.060]) * hor[..., None]
    img = (img * fade + vio * (1 - fade)).astype(np.float32)
    fade = 1.0
    # the stars
    sd = sc.stars
    zc = sd @ fwd
    ok = zc > 0.1
    x = (w / 2 + f_px * (sd[ok] @ right) / zc[ok]).astype(int)
    y = (h / 2 - f_px * (sd[ok] @ up) / zc[ok]).astype(int)
    inb = (x >= 0) & (x < w) & (y >= 0) & (y < h)
    img[y[inb], x[inb]] += sc.star_b[ok][inb][:, None] * np.array([0.5, 0.55, 0.7]) * fade
    return img


DEMOS = _work("demos", "")
SHIRT_INK = [(1.0, 1.0, 1.0), (1.0, 0.85, 0.1), (0.2, 0.95, 1.0), (1.0, 0.35, 0.75), (0.5, 1.0, 0.3)]
SLOT = BAR          # seconds a picture holds on the screens: a bar


def load_demos(callouts):
    """The callouts' pictures (the licence audit, callouts_final.json): the 8 demos we may show play their footage
    (DEMOS/<slug>/frames_160x120.rgb, 12 fps; their title screens on the shirts); the other 55 are parodies, a stand-in
    effect worked out each frame (`standin_frame`) under the parody's title and group, never footage."""
    out = []
    for e in load_callout_entries():
        fr, logo = None, None
        if e["kind"] == "clip":
            raw = np.fromfile(WORK_DEMOS + e["footage"], np.uint8)
            fr = raw[:len(raw) // 57600 * 57600].reshape(-1, 120, 160, 3)
            folder = DEMOS + e["slug"]
            src = {}
            if os.path.exists(folder + "/logo_source.json"):
                import json
                src = json.load(open(folder + "/logo_source.json"))
            if src.get("kind") != "frame from footage" and os.path.exists(folder + "/logo.png"):
                logo = Image.open(folder + "/logo.png").convert("RGBA")
        out.append(dict(title=e["title"], group=e["group"], frames=fr, logo=logo,
                        kind=STANDINS.index(e["standin"]) if e["kind"] == "parody" else -1,
                        hue=standin_hue(e["title"]), stars=standin_stars(e["title"]),
                        caption=caption_image(e["title"], e["group"])))
    print(f"demos: {sum(d['frames'] is not None for d in out)} clips, "
          f"{sum(d['frames'] is None for d in out)} parodies (stand-ins)", file=sys.stderr)
    return out


def logo_for(demos, group):
    for d in demos:
        if d["group"] == group and d["logo"] is not None:
            return d["logo"]
    return None


def clips_of(demos):
    """The demos with footage (the giant screen's montage cycles only these), in the list's order."""
    return [d for d in demos if d["frames"] is not None]


def demo_frame(d, t):
    """A demo's picture at t (160x120, linear): its footage, or its parody's stand-in effect."""
    if d["frames"] is not None:
        return srgb_lin(d["frames"][int(t * FPS) % len(d["frames"])].astype(np.float32) / 255)
    f = standin_frame(d, int(round(t * FPS)))
    return srgb_lin(f.repeat(2, 0).repeat(2, 1).astype(np.float32) / 255)


# ---- the parodies' stand-ins: classic effects worked out at 80 x 60 (as CARDIAC does them: printed tables, a few
# sums a texel), shown at 160 x 120 ---------------------------------------------------------------------------------

STANDINS = ["plasma", "tunnel", "stars", "rotozoom", "rings"]
SW, SH = 80, 60


def standin_hue(title):
    """The stand-in's colours: where on the colour wheel it starts (0..99), from its title (a fixed hash)."""
    import zlib
    return zlib.crc32(title.encode()) % 100


def standin_stars(title):
    """The starfield's 300 stars (x and y across the screen, in thousandths of 30 texels; where along the 2 units of
    depth, thousandths), from its title's length (as the design had them)."""
    st = np.random.default_rng(len(title)).uniform(-1, 1, (300, 3))
    return np.stack([np.round(st[:, 0] * 30000), np.round(st[:, 1] * 30000), np.round((st[:, 2] + 1) * 1000) % 2000],
                    1).astype(np.int64)


_SD = {}


def standin_tables():
    """The stand-ins' printed numbers: per texel (80 x 60, row by row) the plasma's four places (256ths of a turn),
    the tunnel's depth and angle (256ths) and its fade (0..255, times 3), the place in fiftieths (x, y) and its squares
    for the rings; the tables: a sine in 0..63 (512 places), half on / half off, the tunnel's value, the rotozoomer's
    parity and value, the rings' radius (256ths of a turn) by the squared distance, the rings' product, the palette
    (by the hue 0..99 and the value 0..255, sRGB bytes)."""
    if _SD:
        return _SD
    w, u = np.mgrid[0:SH, 0:SW].astype(np.float64)
    x, y = (u + 0.5 - 40) / 20, (w + 0.5 - 30) / 20
    r = np.hypot(x, y)
    turn = lambda a: np.floor(np.mod(a / (2 * math.pi), 1.0) * 256).astype(np.int64) % 256
    d = {}
    d["pl"] = np.stack([turn(x * 3), turn(y * 4), turn((x + y) * 2.5), turn(r * 5)]).reshape(4, -1)
    rr = r + 1e-3
    d["tu"] = np.stack([np.floor(np.mod(1.0 / rr / 0.5, 1.0) * 256).astype(np.int64) % 256,
                        np.floor(np.mod(np.arctan2(y, x) / math.pi * 4, 1.0) * 256).astype(np.int64) % 256,
                        np.round(np.clip(rr, 0, 1) * 255).astype(np.int64) * 3]).reshape(3, -1)
    x50, y50 = np.round(x * 50).astype(np.int64), np.round(y * 50).astype(np.int64)
    d["xy"] = np.stack([x50, y50, y50 * y50, (y50 - 25) ** 2]).reshape(4, -1)
    i = np.arange(512)
    d["sin64"] = np.clip(np.floor((np.sin(2 * math.pi * i / 256) + 1) * 32), 0, 63).astype(np.int64)
    d["half"] = ((i % 256) < 128).astype(np.int64)
    k = np.arange(768)
    d["tun"] = np.where(k % 3 == 1, k // 3, 0)
    d["par"] = np.arange(-64, 65) % 2                  # from -64
    d["rot"] = np.array([26, 230, 26])
    q = np.arange(40_000)
    d["rsq"] = np.floor(np.mod(np.sqrt(q) / 50 * 9 / (2 * math.pi), 1.0) * 256).astype(np.int64) % 256
    s1 = (np.arange(64) + 0.5) / 32 - 1
    d["ring"] = np.round((0.5 + 0.5 * s1[:, None] * s1[None, :]) * 255).astype(np.int64).reshape(-1)
    d["sq"] = np.arange(-400, 401) ** 2                 # from -400
    hue = np.arange(100)[:, None] / 100 + np.arange(256)[None, :] / 255 * 0.6
    v = np.arange(256)[None, :] / 255
    d["pal"] = np.stack([np.round((0.5 + 0.5 * np.cos(2 * math.pi * (hue + c / 3))) * v * 255) for c in range(3)],
                        -1).astype(np.int64)            # (100, 256, 3)
    d["vb"] = np.round(np.clip(1.2 - (np.arange(2050) / 1000) / 2, 0.2, 1) * 255).astype(np.int64)
    _SD.update(d)
    return _SD


def standin_values(kind, hue, stars, t12):
    """The stand-in's value (0..255) at every texel at the show's twelfth t12, as CARDIAC works it out."""
    d = standin_tables()
    ph = lambda rad_s: int(np.floor(np.mod(rad_s * t12 / 12 / (2 * math.pi), 1.0) * 256)) % 256
    if kind == 0:          # plasma: four sines along x, y, the diagonal and out from the middle, moving
        tt = [ph(2.0), ph(-1.5), ph(1.0), ph(-3.0)]
        return sum(d["sin64"][d["pl"][k] + tt[k]] for k in range(4))
    if kind == 1:          # tunnel: rings rushing in, crossed with spokes turning, dark in the middle
        b = d["half"][d["tu"][0] + ph(6 * math.pi)] + d["half"][d["tu"][1] + ph(math.pi)]
        return d["tun"][b + d["tu"][2]]
    if kind == 2:          # stars: 300 flying at the viewer
        v = np.zeros(SW * SH, np.int64)
        zq = np.mod(stars[:, 2] - t12 * 50, 2000) + 50
        sx = 40 + np.floor(stars[:, 0] / zq).astype(np.int64)
        sy = 30 + np.floor(stars[:, 1] / zq).astype(np.int64)
        for k in np.nonzero((sx >= 0) & (sx < SW) & (sy >= 0) & (sy < SH))[0]:
            v[sy[k] * SW + sx[k]] = d["vb"][zq[k]]
        return v
    x50, y50, y2a, y2b = d["xy"]
    if kind == 3:          # rotozoomer: a checkerboard turning and zooming
        a, z = 0.7 * t12 / 12, 1500 + round(math.sin(t12 / 12) * 999)          # thousandths
        cz, sz = math.floor(round(math.cos(a) * 999) * z / 1000), math.floor(round(math.sin(a) * 999) * z / 1000)
        p = np.floor_divide(x50 * cz - y50 * sz, 25000)
        q = np.floor_divide(x50 * sz + y50 * cz, 25000)
        return d["rot"][d["par"][p + 64] + d["par"][q + 64]]
    # rings: two sets of rings from moving centres, multiplied (moire)
    s50 = math.floor(round(math.sin(t12 / 12) * 999) * 50 / 999)
    q1 = d["sin64"][d["rsq"][d["sq"][x50 - s50 + 400] + y2a] + ph(-4.0)]
    q2 = d["sin64"][d["rsq"][d["sq"][x50 + s50 + 400] + y2b]]
    return d["ring"][q1 * 64 + q2]


def standin_frame(dm, t12):
    """A parody's stand-in at t12: 60 x 80 sRGB bytes, its value through its palette."""
    v = standin_values(dm["kind"], dm["hue"], dm["stars"], t12)
    return standin_tables()["pal"][dm["hue"], v].reshape(SH, SW, 3).astype(np.uint8)


# ---- emblems beside a parody group's name (callouts_final.json "emblem"): bold silhouettes drawn here, no fonts ----

# a horse's head (facing left, with its neck and mane), in a unit square (x right, y down); its eye cut out
HORSE = [(0.56, 0.00), (0.47, 0.17), (0.37, 0.22), (0.24, 0.36), (0.07, 0.60), (0.01, 0.71), (0.04, 0.79),
         (0.13, 0.83), (0.24, 0.80), (0.37, 0.68), (0.47, 0.78), (0.53, 1.00), (1.00, 1.00), (0.93, 0.70),
         (0.85, 0.45), (0.74, 0.20), (0.67, 0.02), (0.62, 0.11)]
HORSE_EYE = (0.36, 0.36, 0.045)


def emblem_mask(kind, size):
    """An emblem as a 0..1 mask, size x size (drawn 4 times as big, then averaged: soft edges)."""
    if not kind:
        return None
    k = 4 * size
    im = Image.new("L", (k, k), 0)
    d = ImageDraw.Draw(im)
    if kind == "horse":
        d.polygon([(x * (k - 1), y * (k - 1)) for x, y in HORSE], fill=255)
        ex, ey, er = HORSE_EYE
        d.ellipse([(ex - er) * k, (ey - er) * k, (ex + er) * k, (ey + er) * k], fill=0)
    else:
        raise ValueError(f"no emblem {kind!r}")
    return np.asarray(im.resize((size, size), Image.BOX)).astype(np.float32) / 255


def put_emblem(im, kind, x, y, size, colour):
    """Lays the emblem (its top-left at x, y) over an RGB(A) picture in a colour."""
    m = emblem_mask(kind, size)
    if m is None:
        return
    a = np.asarray(im).astype(np.float32)
    c = np.array(colour, np.float32)[:a.shape[-1]]
    x, y = int(round(x)), int(round(y))
    h_, w_ = a.shape[:2]
    y0, y1, x0, x1 = max(y, 0), min(y + size, h_), max(x, 0), min(x + size, w_)
    mm = m[y0 - y:y1 - y, x0 - x:x1 - x, None]
    a[y0:y1, x0:x1] = a[y0:y1, x0:x1] * (1 - mm) + c * mm
    im.paste(Image.fromarray(np.round(a).astype(np.uint8), im.mode))


def emblem_of(group):
    """The emblem by a group's name (the list's), or None."""
    for e in load_callout_entries():
        if e["group"] == group and e.get("emblem"):
            return e["emblem"]
    return None


def caption_image(title, group, w=384, h=40):
    """A caption: TITLE / GROUP on one line, white and yellow on black (the group's emblem before its name)."""
    from PIL import ImageFont
    em = emblem_of(group)
    im = Image.new("RGB", (w, h), (0, 0, 0))
    d = ImageDraw.Draw(im)
    sup = "/System/Library/Fonts/Supplemental/"
    size = 34
    while size > 12:
        f1 = ImageFont.truetype(sup + "Impact.ttf", size)
        f2 = ImageFont.truetype(sup + "DIN Condensed Bold.ttf", size)
        a = d.textlength(title.upper() + "  ", font=f1)
        es = int(size * 1.05) if em else 0            # the emblem before the group's name, its size the type's
        b = d.textlength(group.upper(), font=f2) + (es + size * 0.2 if em else 0)
        if a + b <= w - 16:
            break
        size -= 2
    x0 = (w - a - b) / 2
    d.text((x0, h / 2), title.upper(), font=f1, fill=(255, 255, 255), anchor="lm")
    if em:
        put_emblem(im, em, x0 + a, h / 2 - es / 2, es, (255, 220, 90))
        d = ImageDraw.Draw(im)
    d.text((x0 + a + (es + size * 0.2 if em else 0), h / 2 + 2), group.upper(), font=f2, fill=(255, 220, 90), anchor="lm")
    return srgb_lin(np.asarray(im).astype(np.float32) / 255)


def shirt_image(group, ink, logo, w=96, h=100):
    """A back print: the group's logo if there is one, else its name, big; RGBA (the ink and where it is)."""
    from PIL import ImageFont
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    if logo is not None:
        lg = logo.copy()
        lg.thumbnail((w - 8, h - 8))
        im.paste(lg, ((w - lg.width) // 2, (h - lg.height) // 2), lg)
        a = np.asarray(im).astype(np.float32) / 255
        lum = a[..., :3].max(-1, keepdims=True)
        a[..., 3:] = a[..., 3:] * np.clip((lum - 0.12) * 4, 0, 1)      # the title screen's black drops out
        a[..., :3] = np.clip(a[..., :3] * 1.3, 0, 1)
        return np.concatenate([srgb_lin(a[..., :3]), a[..., 3:]], -1)
    em = emblem_of(group)
    if em:              # the emblem big over the name
        ink255 = tuple(int(c * 255) for c in ink) + (255,)
        put_emblem(im, em, (w - 52) / 2, 4, 52, ink255)
        d = ImageDraw.Draw(im)
        h0 = 58
    else:
        h0 = 0
    words = group.upper().replace("&", "+").split()
    lines, cur = [], ""
    for wd in words:
        if len(cur) + len(wd) > 9 and cur:
            lines.append(cur)
            cur = wd
        else:
            cur = (cur + " " + wd).strip()
    lines.append(cur)
    lines = lines[:3]
    sup = "/System/Library/Fonts/Supplemental/"
    size = 60
    while size > 10:
        f = ImageFont.truetype(sup + "Impact.ttf", size)
        wid = max(d.textlength(l_, font=f) for l_ in lines)
        if wid <= w - 6 and size * 1.05 * len(lines) <= h - h0 - 6:
            break
        size -= 2
    y = h0 + (h - h0 - size * 1.05 * len(lines)) / 2
    for l_ in lines:
        d.text((w / 2, y + size * 0.52), l_, font=f, fill=tuple(int(c * 255) for c in ink) + (255,), anchor="mm")
        y += size * 1.05
    a = np.asarray(im).astype(np.float32) / 255
    return np.concatenate([srgb_lin(a[..., :3]), a[..., 3:]], -1)


def on_screen(sc, t, which):
    """Which demo a screen shows in this slot (each screen its own order)."""
    slot = int(t // SLOT)
    return sc.demos[(slot * 3 + which * 7) % len(sc.demos)]


CARDIAC_BIN = os.path.join(HERE, "../segments/ending/data/cardiac.bin")    # the photo of the CARDIAC
_PH = {}


def _words(lines, w, h):
    """Words on black: (text, centre x, centre y, size, colour, face) in the picture's pixels."""
    from PIL import ImageFont
    im = Image.new("RGB", (w, h), (0, 0, 0))
    d = ImageDraw.Draw(im)
    for text, x, y, size, colour, face in lines:
        f = ImageFont.truetype("/System/Library/Fonts/Supplemental/" + face, size)
        d.text((x, y), text, font=f, fill=colour, anchor="mm")
    return im


def thanks_fingerman(t):
    """The airship's screen (192x108) in its hold: THANK YOU / SAUL FINGERMAN, type only."""
    if "fing" not in _PH:
        im = _words([("THANK YOU", 96, 24, 26, (255, 255, 255), "Impact.ttf"),
                     ("SAUL FINGERMAN", 96, 58, 30, (255, 255, 255), "Impact.ttf"),
                     ("CO-CREATORS OF CARDIAC, BELL LABS 1968", 96, 90, 12, (255, 220, 90),
                      "DIN Condensed Bold.ttf")], 192, 108)
        _PH["fing"] = srgb_lin(np.asarray(im).astype(np.float32) / 255)
    return _PH["fing"]


def thanks_hagelbarger(t):
    """The giant screen (384x192): THANK YOU / D. W. HAGELBARGER, type only, big."""
    if "hag" not in _PH:
        im = _words([("THANK YOU", 192, 52, 48, (255, 255, 255), "Impact.ttf"),
                     ("D. W. HAGELBARGER", 192, 110, 48, (255, 255, 255), "Impact.ttf"),
                     ("CO-CREATORS OF CARDIAC, BELL LABS 1968", 192, 162, 20, (255, 220, 90),
                      "DIN Condensed Bold.ttf")], 384, 192)
        _PH["hag"] = srgb_lin(np.asarray(im).astype(np.float32) / 255)
    return _PH["hag"]


def cardiac_picture():
    """The giant screen's last picture: the CARDIAC (the ending's cut-out), big, on black."""
    if "card" not in _PH:
        raw = np.fromfile(CARDIAC_BIN, np.uint8).reshape(648, 679, 4).astype(np.float32)
        rgb = raw[..., :3] / 255 * (raw[..., 3:] / 10)
        ph = Image.fromarray((rgb * 255).astype(np.uint8)).resize((201, 192), Image.LANCZOS)
        im = Image.new("RGB", (384, 192), (0, 0, 0))
        im.paste(ph, ((384 - 201) // 2, 0))
        _PH["card"] = srgb_lin(np.asarray(im).astype(np.float32) / 255) * 1.2
    return _PH["card"]


def screen_picture(sc, t):
    if t >= CARDIAC_AT:
        return cardiac_picture()
    if HAG_AT[0] <= t < HAG_AT[1]:
        return thanks_hagelbarger(t)
    return _screen_demo(sc, t)


def _screen_demo(sc, t):
    """The giant screen (384x192): a montage of the 8 demos we may show (the clips), a different one every 2 beats
    (on the beat's clock, from BEAT_T0), round them in the callouts' order; each shown 4:3 in the middle, playing its clip at its own time,
    with its title and group along the bottom."""
    chunk = math.floor((t - BEAT_T0) / CHUNK)
    clips = clips_of(sc.demos)
    d = clips[chunk % len(clips)]
    img = np.zeros((192, 384, 3), np.float32)
    f = demo_frame(d, t)
    big = np.asarray(Image.fromarray((np.clip(f, 0, 1) ** (1 / 2.2) * 255).astype(np.uint8)).resize((256, 192),
                                                                                              Image.NEAREST))
    img[:, 64:320] = srgb_lin(big.astype(np.float32) / 255)
    img[150:190, :] = d["caption"]
    return img


def led_picture(sc, t, i):
    """The LED walls (120x220, portrait): a demo running at the top, its title and group in big type under it."""
    d = on_screen(sc, t, i + 1)
    img = np.zeros((220, 120, 3), np.float32)
    f = demo_frame(d, t)
    img[10:100] = np.asarray(Image.fromarray((np.clip(f, 0, 1) ** (1 / 2.2) * 255).astype(np.uint8)).resize(
        (120, 90), Image.NEAREST)).astype(np.float32) / 255
    img[10:100] = srgb_lin(img[10:100])
    c = vertical_caption(d["title"], d["group"])
    img[110:110 + c.shape[0]] = c
    pulse = beat_pulse(t, 3.0)
    return img * (1.0 + 0.6 * pulse)


_VC = {}


def vertical_caption(title, group, w=120, h=100):
    key = (title, group)
    if key in _VC:
        return _VC[key]
    from PIL import ImageFont
    im = Image.new("RGB", (w, h), (0, 0, 0))
    d = ImageDraw.Draw(im)
    sup = "/System/Library/Fonts/Supplemental/"
    size = 40
    while size > 10:
        f1 = ImageFont.truetype(sup + "Impact.ttf", size)
        if d.textlength(title.upper(), font=f1) <= w - 6:
            break
        size -= 2
    em = emblem_of(group)
    gs = 26
    while gs > 9:
        f2 = ImageFont.truetype(sup + "DIN Condensed Bold.ttf", gs)
        es = int(gs * 1.1) if em else 0
        if d.textlength(group.upper(), font=f2) + (es + 3 if em else 0) <= w - 6:
            break
        gs -= 1
    d.text((w / 2, 30), title.upper(), font=f1, fill=(255, 255, 255), anchor="mm")
    gw = d.textlength(group.upper(), font=f2) + (es + 3 if em else 0)
    if em:
        put_emblem(im, em, (w - gw) / 2, 70 - es / 2, es, (255, 220, 90))
        d = ImageDraw.Draw(im)
    d.text(((w - gw) / 2 + (es + 3 if em else 0), 70), group.upper(), font=f2, fill=(255, 220, 90), anchor="lm")
    _VC[key] = srgb_lin(np.asarray(im).astype(np.float32) / 255)
    return _VC[key]


def banner_picture(sc, t, q):
    """The LED banners round the bowl (160x100): a demo letterboxed, its caption under it."""
    d = on_screen(sc, t, q + 3)
    img = np.zeros((100, 160, 3), np.float32)
    f = demo_frame(d, t)
    img[0:75, 30:130] = f[np.linspace(0, 119, 75).astype(int)][:, np.linspace(0, 159, 100).astype(int)]
    cap = d["caption"]
    img[78:98] = cap[np.linspace(0, cap.shape[0] - 1, 20).astype(int)][:, np.linspace(0, cap.shape[1] - 1,
                                                                                         160).astype(int)]
    return img * 1.4


def light_at(sc, t, p, n, vd, pic, lw_pics, airship=False):
    """The light falling on points p with normals n: the screen and the LED walls (as grids of small area lights),
    the moving heads' pools, the pyro."""
    L = np.zeros_like(p)
    s = sc.screen
    c0 = np.array(s["centre"])
    gh, gw = 3, 6
    ph_, pw_ = pic.shape[:2]
    sub_area = s["width"] * s["height"] / (gh * gw)
    for iy in range(gh):
        for ix in range(gw):
            blk = pic[iy * ph_ // gh:(iy + 1) * ph_ // gh, ix * pw_ // gw:(ix + 1) * pw_ // gw].mean((0, 1))
            q = c0 + np.array([(ix + 0.5) / gw - 0.5, 0, 0.5 - (iy + 0.5) / gh]) * [s["width"], 0, s["height"]]
            L += area_light(p, n, q, np.array([0, -1.0, 0]), sub_area, blk * 0.6)
    for i, lw in enumerate(sc.info["led_walls"]):
        L += area_light(p, n, np.array(lw["centre"]), np.array([0, -1.0, 0]), lw["width"] * lw["height"],
                        lw_pics[i].mean((0, 1)) * 0.3)
    sh = show(t)
    if sh["strobe"] > 0:          # the strobes on the stage's truss
        for q in ([-30, 44, 40], [0, 44, 40], [30, 44, 40]):
            v = np.array(q, float) - p
            dd = np.linalg.norm(v, axis=1)
            ndl = np.maximum((n * v).sum(1) / (dd + 1e-9), 0)
            L += np.array([0.9, 0.92, 1.0]) * (sh["strobe"] * 9000.0 * ndl / (dd * dd + 100))[:, None]
    for l in sc.spots:
        d = spot_dir(l, t)
        q = np.array(l["pos"])
        v = q - p
        dd = np.linalg.norm(v, axis=1)
        v /= dd[:, None] + 1e-9
        cosang = -(v @ d)
        cone = np.exp(-np.maximum(1 - cosang, 0) / (1 - math.cos(math.radians(l["cone"]))))
        ndl = np.maximum((n * v).sum(1), 0) * 0.7 + 0.3 * np.maximum((n * v).sum(1) + 0.4, 0)
        L += np.array(l["rgb"]) * (sh["spots"] * 9000.0 * cone * ndl / (dd * dd + 1))[:, None]
    if airship:                   # the searchlights that have caught it: pools of white on the hull
        for l in sc.search:
            if not l.get("aim"):
                continue
            d = search_dir(l, t, sc)
            v = np.array(l["pos"]) - p
            dd = np.linalg.norm(v, axis=1)
            v /= dd[:, None] + 1e-9
            cosang = -(v @ d)
            cone = np.exp(-np.maximum(1 - cosang, 0) / (1 - math.cos(math.radians(8.0))))
            ndl = np.maximum((n * v).sum(1), 0)
            L += np.array(l["rgb"]) * (cone * ndl * 35.0)[:, None]
        return L
    py = pyro_level(t)
    if py > 0:
        for q in sc.info["pyro"]:
            v = np.array(q) + [0, 0, 6] - p
            dd = np.linalg.norm(v, axis=1)
            ndl = np.maximum((n * v).sum(1) / (dd + 1e-9), 0)
            L += np.array([1.0, 0.55, 0.2]) * (py * 2500.0 * ndl / (dd * dd + 20))[:, None]
    return L


def area_light(p, n, q, qn, area, rgb):
    v = q - p
    d2 = (v * v).sum(1)
    d = np.sqrt(d2) + 1e-9
    v = v / d[:, None]
    ndl = np.maximum((n * v).sum(1), 0)
    cs = np.maximum(-(v @ qn), 0)
    return rgb[None, :] * (ndl * cs * area / (math.pi * d2 + area))[:, None]


def project(pts, eye, fwd, right, up, f_px, w, h):
    rel = pts - eye
    zc = rel @ fwd
    x = w / 2 + f_px * (rel @ right) / np.maximum(zc, 1e-6)
    y = h / 2 - f_px * (rel @ up) / np.maximum(zc, 1e-6)
    return x, y, zc


def splat(img, depth, x, y, zc, rgb, size):
    """Additive round dots, hidden behind nearer surfaces; size in pixels (per dot or one)."""
    h, w = depth.shape
    size = np.broadcast_to(np.asarray(size, float), x.shape)
    ok = (zc > 0.2) & (x > -4) & (x < w + 4) & (y > -4) & (y < h + 4)
    x, y, zc, rgb, size = x[ok], y[ok], zc[ok], rgb[ok], size[ok]
    r = np.maximum(size, 0.5)
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            xi, yi = np.floor(x).astype(int) + dx, np.floor(y).astype(int) + dy
            inb = (xi >= 0) & (xi < w) & (yi >= 0) & (yi < h)
            dd = np.hypot(xi + 0.5 - x, yi + 0.5 - y)
            wgt = np.clip(r + 0.5 - dd, 0, 1)
            vis = inb.copy()
            vis[inb] = depth[yi[inb], xi[inb]] > zc[inb] * 0.995
            np.add.at(img, (yi[vis], xi[vis]), rgb[vis] * wgt[vis][:, None])


def points(img, depth, pts, rgb, eye, fwd, right, up, f_px, size=1.0):
    h, w = depth.shape
    x, y, zc = project(pts, eye, fwd, right, up, f_px, w, h)
    splat(img, depth, x, y, zc, rgb, size)


def phones(sc, img, depth, mv, ph, is_near, sprite_rg, lift, eye, fwd, right, up, f_px, t):
    """A phone's screen in each raised right hand: a small cold light."""
    h, w = depth.shape
    ids = np.nonzero(mv == MOVES.index("raise"))[0]
    if len(ids) == 0:
        return
    k_at = (sc.cmodel[ids] * len(MOVES) + mv[ids]) * NPHASE + ph[ids]
    hand = sc.atlas_hand[k_at, 0]
    pos = np.stack([sc.cx[ids], sc.cy[ids], sc.cz[ids]], 1)
    # the hand across and up, in the person's own frame (facing) for the near, the sprite's for the far
    a = sc.cface[ids] - math.pi / 2
    rg_face = np.stack([np.cos(a), np.sin(a), np.zeros(len(ids))], 1)
    rg = np.where(is_near[ids][:, None], rg_face, sprite_rg[ids])
    rg = np.where((np.abs(rg).sum(1) < 1e-6)[:, None], rg_face, rg)
    p = pos + rg * (hand[:, 0:1] * sc.cscale[ids][:, None]) + np.c_[np.zeros((len(ids), 2)),
                                                                     hand[:, 1] * sc.cscale[ids]]
    p = p + (eye - p) / (np.linalg.norm(eye - p, axis=1, keepdims=True) + 1e-9) * 0.15
    x, y, zc = project(p, eye, fwd, right, up, f_px, w, h)
    fl = 0.8 + 0.2 * np.sin(ids * 1.7 + t * 5)
    rgb = np.array([0.75, 0.85, 1.0]) * (1.6 * fl)[:, None]
    size = np.clip(f_px * 0.07 / np.maximum(zc, 1e-3), 0.55, 4.0)
    splat(img, depth, x, y, zc, rgb.astype(np.float32), size)


def photo_flashes(sc, img, depth, eye, fwd, right, up, f_px, t):
    """People taking pictures: a few dozen flashes a frame, from anyone in the crowd, a frame each, a colour each;
    nothing to do with the beat."""
    h, w = depth.shape
    rng = np.random.default_rng((int(round(t * FPS)) + 100000) * 131 + 11)   # (t is negative before the crest)
    ids = rng.integers(0, len(sc.cx), 48)
    p = np.stack([sc.cx[ids], sc.cy[ids], sc.cz[ids] + 1.8 * sc.cscale[ids]], 1)
    p = p + (eye - p) / (np.linalg.norm(eye - p, axis=1, keepdims=True) + 1e-9) * 0.3
    x, y, zc = project(p, eye, fwd, right, up, f_px, w, h)
    hue = rng.uniform(0, 1, (len(ids), 3)) ** 2
    rgb = (0.3 + 0.7 * hue / hue.max(1, keepdims=True)) * 9.0
    size = np.clip(f_px * 0.35 / np.maximum(zc, 1e-3), 1.0, 7.0)
    splat(img, depth, x, y, zc, rgb.astype(np.float32), size)


def confetti(sc, img, depth, eye, fwd, right, up, f_px, t):
    h, w = depth.shape
    since = t - (JOIN_T - 0.3) + 1.5          # the cannons fired just before the crest: it's in the air at the reveal
    z = 55 - (sc.conf_p[:, 2] * 55 + since * sc.conf_fall * 2.4) % 55
    x = sc.conf_p[:, 0] + 1.5 * np.sin(since * 1.3 + sc.conf_ph)
    y = sc.conf_p[:, 1] + 1.0 * np.cos(since * 1.1 + sc.conf_ph)
    p = np.stack([x, y, z + 2], 1)
    flick = 0.5 + 0.5 * np.sin(since * 9 + sc.conf_ph * 3) ** 2
    px, py, zc = project(p, eye, fwd, right, up, f_px, w, h)
    rgb = sc.conf_c * (1.1 * flick)[:, None]
    size = np.clip(f_px * 0.05 / np.maximum(zc, 1e-3), 0.5, 3.0)
    splat(img, depth, px, py, zc, rgb.astype(np.float32), size)


def beams(sc, t):
    """Every beam in the haze as an axis: (origin, way, length, width at the source, spread, colour and strength,
    fall-off distance)."""
    out = []
    sh = show(t)
    for l in sc.spots:
        u = spot_dir(l, t)
        o = np.array(l["pos"], float)
        L = (o[2] - 1.8) / max(-u[2], 0.05)                 # down to the crowd's heads
        out.append((o, u, L, 0.2, math.tan(math.radians(l["cone"] * 0.25)), np.array(l["rgb"]) * 0.7 * sh["spots"], 90.0))
    for l in sc.search:
        u = search_dir(l, t, sc)
        lev = 1.0 if l.get("aim") else sh["search"]       # the ones on the airship stay on it
        out.append((np.array(l["pos"], float), u, 2500.0, 0.6, math.tan(math.radians(1.2 if l.get("aim") else 0.7)),
                    np.array(l["rgb"]) * 0.55 * l["power"] * lev, 900.0))
    return out


def volumetric(sc, depth, eye, fwd, right, up, f_px, w, h, t):
    """The beams in the haze, worked out per ray at half size: each beam's light where the ray passes closest to
    its axis, a Gaussian across it, widening from the source; hidden where a surface is nearer."""
    q = 2
    hw, hh = w // q, h // q
    ys, xs = np.mgrid[0:hh, 0:hw]
    dx = ((xs + 0.5) * q - w / 2) / f_px
    dy = -((ys + 0.5) * q - h / 2) / f_px
    d = fwd[None, None] + dx[..., None] * right[None, None] + dy[..., None] * up[None, None]
    dn = np.linalg.norm(d, axis=-1)
    d = (d / dn[..., None]).reshape(-1, 3)
    dmax = (depth.reshape(hh, q, hw, q).min((1, 3)).reshape(-1) * dn.reshape(-1))
    acc = np.zeros((hw * hh, 3), np.float32)
    for o, u, L, s0, spread, rgb, fall in beams(sc, t):
        w0 = eye - o
        bdot = d @ u
        dd_ = d @ w0
        e = float(u @ w0)
        den = np.maximum(1 - bdot * bdot, 0.04)       # a beam seen end on: held, not a blinding blob
        sb = np.clip((e - bdot * dd_) / den, 0, L)
        p = o[None, :] + sb[:, None] * u[None, :]
        tr = np.maximum(((p - eye) * d).sum(1), 0)
        dist = np.linalg.norm(eye[None, :] + tr[:, None] * d - p, axis=1)
        sig = s0 + spread * sb
        haze = np.exp(-np.maximum(p[:, 2] - 45, 0) / 400.0)
        vis = np.clip((dmax - tr) / sig + 1, 0, 1)
        val = np.exp(-0.5 * (dist / sig) ** 2) * 2.5 / np.sqrt(den) / (1 + sig)
        val = val * haze * vis / (1 + sb / fall) ** 2
        acc += (val[:, None] * rgb[None, :]).astype(np.float32)
    acc = acc.reshape(hh, hw, 3)
    out = np.zeros((h, w, 3), np.float32)
    for c in range(3):
        out[..., c] = np.asarray(Image.fromarray(acc[..., c]).resize((w, h), Image.BILINEAR))
    return out


def lasers(sc, img, depth, eye, fwd, right, up, f_px, t):
    """Each beam a line from its source 400 m out: a bright core and a glow, hidden behind nearer surfaces."""
    h, w = depth.shape
    ys, xs = np.mgrid[0:h, 0:w]
    xs = xs + 0.5
    ys = ys + 0.5
    lev = show(t)["lasers"]
    for l in sc.lasers:
        src = np.array(l["pos"], float)
        dirs, on = laser_beams(l, t)
        for dvec, o in zip(dirs, on):
            if o < 0.02:
                continue
            a, b_ = src, src + dvec * 125.0
            # cut the beam at the near plane
            za, zb = (a - eye) @ fwd, (b_ - eye) @ fwd
            nearz = 0.5
            if za < nearz and zb < nearz:
                continue
            if za < nearz:
                a = a + (b_ - a) * (nearz - za) / (zb - za)
                za = nearz
            if zb < nearz:
                b_ = a + (b_ - a) * (nearz - za) / (zb - za)
                zb = nearz
            ax, ay = w / 2 + f_px * ((a - eye) @ right) / za, h / 2 - f_px * ((a - eye) @ up) / za
            bx, by = w / 2 + f_px * ((b_ - eye) @ right) / zb, h / 2 - f_px * ((b_ - eye) @ up) / zb
            x0, x1 = int(max(min(ax, bx) - 12, 0)), int(min(max(ax, bx) + 12, w))
            y0, y1 = int(max(min(ay, by) - 12, 0)), int(min(max(ay, by) + 12, h))
            if x1 <= x0 or y1 <= y0:
                continue
            X, Y = xs[y0:y1, x0:x1], ys[y0:y1, x0:x1]
            vx, vy = bx - ax, by - ay
            ll = vx * vx + vy * vy + 1e-9
            s = np.clip(((X - ax) * vx + (Y - ay) * vy) / ll, 0, 1)
            dist = np.hypot(X - (ax + s * vx), Y - (ay + s * vy))
            # the depth along the beam at s (1/z is linear on the screen)
            zl = 1.0 / ((1 - s) / za + s / zb)
            vis = depth[y0:y1, x0:x1] > zl
            fade = np.exp(-zl / 500.0) * (1 - 0.7 * s)
            inten = (np.exp(-(dist / 0.9) ** 2) * 1.6 + np.exp(-dist / 5.0) * 0.25) * vis * fade * o
            img[y0:y1, x0:x1] += inten[..., None] * np.array(l["rgb"], np.float32) * 1.4 * lev


def pyro(sc, img, depth, eye, fwd, right, up, f_px, t):
    lev = pyro_level(t)
    if lev <= 0.01:
        return
    h, w = depth.shape
    ys, xs = np.mgrid[0:h, 0:w]
    for i, q in enumerate(sc.info["pyro"]):
        base = np.array(q, float)
        top = base + [0, 0, 14 * (0.6 + 0.4 * lev)]
        za, zb = (base - eye) @ fwd, (top - eye) @ fwd
        if za < 1 or zb < 1:
            continue
        ax, ay = w / 2 + f_px * ((base - eye) @ right) / za, h / 2 - f_px * ((base - eye) @ up) / za
        bx, by = w / 2 + f_px * ((top - eye) @ right) / zb, h / 2 - f_px * ((top - eye) @ up) / zb
        wid = f_px * 2.2 / za
        x0, x1 = int(max(min(ax, bx) - 3 * wid, 0)), int(min(max(ax, bx) + 3 * wid, w))
        y0, y1 = int(max(by - 2 * wid, 0)), int(min(ay + wid, h))
        if x1 <= x0 or y1 <= y0:
            continue
        X, Y = xs[y0:y1, x0:x1] + 0.5, ys[y0:y1, x0:x1] + 0.5
        s = np.clip((Y - ay) / (by - ay + 1e-9), 0, 1)
        cxl = ax + (bx - ax) * s + wid * 0.25 * np.sin(s * 9 + t * 20 + i)
        wl = wid * (1 - 0.7 * s)
        dist = np.abs(X - cxl) / np.maximum(wl, 0.5)
        inten = np.exp(-dist ** 2) * (1 - s) ** 0.5 * lev * ((Y < ay + 2))
        vis = depth[y0:y1, x0:x1] > za * 0.98
        col = np.stack([np.ones_like(s) * 3.0, 1.6 - s * 0.8, 0.5 - s * 0.4], -1)
        img[y0:y1, x0:x1] += (inten * vis)[..., None] * col


def bloom(img):
    h, w = img.shape[:2]
    br = np.maximum(img - 1.1, 0)
    q = 4
    small = br.reshape(h // q, q, w // q, q, 3).mean((1, 3))
    for _ in range(3):
        for ax in (0, 1):
            pad = [(2, 2) if k == ax else (0, 0) for k in range(3)]
            p = np.pad(small, pad, mode="edge")
            n = small.shape[ax]
            small = sum(np.take(p, np.arange(k, k + n), axis=ax) for k in range(5)) / 5
    big = np.repeat(np.repeat(small, q, 0), q, 1)
    return img + 0.8 * big


# ======================================================================================================================
# output

CALLOUTS = _work("callouts_final.json")
WORK_DEMOS = _work("")
PALETTE = [(0.75, 0.08, 0.10), (0.08, 0.25, 0.70), (0.05, 0.45, 0.20), (0.55, 0.10, 0.55), (0.85, 0.45, 0.02),
           (0.05, 0.05, 0.06), (0.02, 0.45, 0.55)]
PALETTE_BALLOON = [(1.0, 0.15, 0.2), (1.0, 0.8, 0.1), (0.2, 0.5, 1.0), (0.2, 0.9, 0.4), (1.0, 0.4, 0.8),
                   (1.0, 1.0, 1.0), (0.6, 0.3, 1.0)]


def load_callout_entries(path=CALLOUTS):
    """The 63 callouts in their order (callouts_final.json, after the licence audit): 8 clips (the demo's own
    title and group, its footage) and 55 parodies (a parody title and group, a stand-in effect)."""
    import json
    return json.load(open(path))["callouts"]


def load_callouts(path=CALLOUTS):
    """(title, group) as shown: the clips' own, the parodies' made-up ones."""
    return [(e["title"], e["group"]) for e in load_callout_entries(path)]


def callout_image(title, group, bg, w=320, h=200):
    """A callout as plain text (no logos): the title big, the group under it, white on a colour."""
    from PIL import ImageFont
    im = Image.new("RGB", (w, h), tuple(int(c * 255) for c in bg))
    d = ImageDraw.Draw(im)
    sup = "/System/Library/Fonts/Supplemental/"
    size = 120
    while size > 20:
        f1 = ImageFont.truetype(sup + "Impact.ttf", size)
        bb = d.textbbox((0, 0), title.upper(), font=f1)
        if bb[2] - bb[0] <= w - 24 and bb[3] - bb[1] <= h * 0.5:
            break
        size -= 4
    em = emblem_of(group)
    gs = 56
    while gs > 14:
        f2 = ImageFont.truetype(sup + "DIN Condensed Bold.ttf", gs)
        gb = d.textbbox((0, 0), group.upper(), font=f2)
        es = int(gs * 1.05) if em else 0
        if gb[2] - gb[0] + (es + gs // 5 if em else 0) <= w - 24:
            break
        gs -= 2
    th = bb[3] - bb[1]
    gh = gb[3] - gb[1]
    y0 = (h - th - gh - 14) // 2
    d.text(((w - (bb[2] - bb[0])) // 2 - bb[0], y0 - bb[1]), title.upper(), font=f1, fill=(255, 255, 255))
    ew = es + gs // 5 if em else 0
    gx = (w - (gb[2] - gb[0]) - ew) // 2
    if em:
        put_emblem(im, em, gx, y0 + th + 14 + gh / 2 - es / 2, es, (255, 230, 120))
        d = ImageDraw.Draw(im)
    d.text((gx + ew - gb[0], y0 + th + 14 - gb[1]), group.upper(), font=f2, fill=(255, 230, 120))
    d.rectangle([0, 0, w - 1, h - 1], outline=(255, 255, 255), width=4)
    return srgb_lin(np.asarray(im).astype(np.float32) / 255)


NATIONS = [("FI", 6), ("SE", 5), ("NO", 4), ("DK", 4), ("IS", 2), ("EE", 2), ("DE", 4), ("NL", 3), ("PL", 3),
           ("HU", 2), ("US", 5), ("GB", 2), ("FR", 1), ("BE", 1), ("CZ", 1), ("SK", 1)]


def national_flag(code, w=320, h=200):
    """A national flag, drawn from its (public domain) design, on the 1.6:1 cloth (the Nordic crosses and the
    tricolours keep their proportions across the width; the stars and the Union Jack simplified a little)."""
    im = Image.new("RGB", (w, h), (255, 255, 255))
    d = ImageDraw.Draw(im)

    def nordic(bg, cross, inner=None, hoist=None, wide=None):
        d.rectangle([0, 0, w, h], fill=bg)
        cx = w * (hoist if hoist else 6.5 / 18)                 # the cross's middle from the hoist
        cw = h * (wide if wide else 3 / 11)
        d.rectangle([cx - cw / 2, 0, cx + cw / 2, h], fill=cross)
        d.rectangle([0, h / 2 - cw / 2, w, h / 2 + cw / 2], fill=cross)
        if inner:
            iw = cw / 2
            d.rectangle([cx - iw / 2, 0, cx + iw / 2, h], fill=inner)
            d.rectangle([0, h / 2 - iw / 2, w, h / 2 + iw / 2], fill=inner)

    def bands(cols, vertical=False):
        k = len(cols)
        for i, c in enumerate(cols):
            if vertical:
                d.rectangle([w * i / k, 0, w * (i + 1) / k, h], fill=c)
            else:
                d.rectangle([0, h * i / k, w, h * (i + 1) / k], fill=c)

    if code == "FI":
        nordic((255, 255, 255), (0, 47, 108))
    elif code == "SE":
        nordic((0, 106, 167), (254, 204, 0), hoist=6.5 / 16, wide=2 / 10)
    elif code == "NO":
        nordic((186, 12, 47), (255, 255, 255), (0, 32, 91), hoist=8 / 22, wide=4 / 16)
    elif code == "DK":
        nordic((200, 16, 46), (255, 255, 255), hoist=14 / 37, wide=4 / 28)
    elif code == "IS":
        nordic((2, 82, 156), (255, 255, 255), (220, 30, 53), hoist=9 / 25, wide=4 / 18)
    elif code == "EE":
        bands([(0, 114, 206), (0, 0, 0), (255, 255, 255)])
    elif code == "DE":
        bands([(0, 0, 0), (221, 0, 0), (255, 206, 0)])
    elif code == "NL":
        bands([(174, 28, 40), (255, 255, 255), (33, 70, 139)])
    elif code == "PL":
        bands([(255, 255, 255), (220, 20, 60)])
    elif code == "HU":
        bands([(205, 42, 62), (255, 255, 255), (67, 111, 77)])
    elif code == "FR":
        bands([(0, 35, 149), (255, 255, 255), (237, 41, 57)], True)
    elif code == "BE":
        bands([(0, 0, 0), (253, 218, 36), (239, 51, 64)], True)
    elif code == "CZ":
        bands([(255, 255, 255), (215, 20, 26)])
        d.polygon([(0, 0), (w / 2, h / 2), (0, h)], fill=(17, 69, 126))
    elif code == "SK":
        bands([(255, 255, 255), (11, 78, 162), (238, 28, 37)])
        x0, y0, sw, sh_ = w * 0.18, h * 0.2, w * 0.2, h * 0.62     # the shield: the double cross on three hills
        d.rounded_rectangle([x0, y0, x0 + sw, y0 + sh_], radius=sw * 0.45, fill=(255, 255, 255))
        d.rounded_rectangle([x0 + 3, y0 + 3, x0 + sw - 3, y0 + sh_ - 3], radius=sw * 0.42, fill=(238, 28, 37))
        cx = x0 + sw / 2
        for hx, hy, hr in ((cx, 0.78, 0.2), (cx - sw * 0.24, 0.84, 0.16), (cx + sw * 0.24, 0.84, 0.16)):  # 3 hills
            d.ellipse([hx - sw * hr, y0 + sh_ * hy, hx + sw * hr, y0 + sh_ * (hy + 0.3)], fill=(11, 78, 162))
        d.rectangle([cx - sw * 0.05, y0 + sh_ * 0.12, cx + sw * 0.05, y0 + sh_ * 0.72], fill=(255, 255, 255))
        d.rectangle([cx - sw * 0.28, y0 + sh_ * 0.24, cx + sw * 0.28, y0 + sh_ * 0.31], fill=(255, 255, 255))
        d.rectangle([cx - sw * 0.36, y0 + sh_ * 0.40, cx + sw * 0.36, y0 + sh_ * 0.48], fill=(255, 255, 255))
    elif code == "US":
        for i in range(13):
            d.rectangle([0, h * i / 13, w, h * (i + 1) / 13], fill=(178, 34, 52) if i % 2 == 0 else (255, 255, 255))
        cw_, ch_ = w * 0.4, h * 7 / 13
        d.rectangle([0, 0, cw_, ch_], fill=(60, 59, 110))
        for r in range(9):
            for c in range(6 if r % 2 == 0 else 5):
                x = cw_ * ((c * 2 + (1 if r % 2 == 0 else 2)) / 12)
                y = ch_ * ((r + 1) / 10)
                rr = h * 0.016
                d.regular_polygon((x, y, rr * 1.4), 5, fill=(255, 255, 255))
    elif code == "GB":
        d.rectangle([0, 0, w, h], fill=(1, 33, 105))
        d.line([(0, 0), (w, h)], fill=(255, 255, 255), width=int(h * 0.2))
        d.line([(0, h), (w, 0)], fill=(255, 255, 255), width=int(h * 0.2))
        d.line([(0, 0), (w, h)], fill=(200, 16, 46), width=int(h * 0.067))
        d.line([(0, h), (w, 0)], fill=(200, 16, 46), width=int(h * 0.067))
        d.rectangle([w / 2 - h / 6, 0, w / 2 + h / 6, h], fill=(255, 255, 255))
        d.rectangle([0, h / 3, w, h * 2 / 3], fill=(255, 255, 255))
        d.rectangle([w / 2 - h / 10, 0, w / 2 + h / 10, h], fill=(200, 16, 46))
        d.rectangle([0, h * 0.4, w, h * 0.6], fill=(200, 16, 46))
    return srgb_lin(np.asarray(im).astype(np.float32) / 255)


_AIRSHIP_TXT = {}


def airship_picture(sc, t):
    """The airship's screen (as the night arrival's): DEMOSPLASH 0x10 in blue over a magenta horizon and an orange
    laser grid racing toward the viewer, cycling with demo clips, 2.5 s each."""
    slot = int(t // 2.5)
    if slot % 3 != 1:      # DEMOSPLASH 0x10 at 2.5-5 s and 10-12.5 s (the camera's look up at it), ...
        return demo_frame(sc.demos[(slot * 5) % len(sc.demos)], t)
    if "txt" not in _AIRSHIP_TXT:
        from PIL import ImageFont
        im = Image.new("L", (160, 120), 0)
        dd = ImageDraw.Draw(im)
        f = ImageFont.truetype("/System/Library/Fonts/Supplemental/Impact.ttf", 30)
        dd.text((80, 34), "DEMOSPLASH", font=f, fill=255, anchor="mm")
        dd.text((80, 66), "0x10", font=f, fill=255, anchor="mm")
        _AIRSHIP_TXT["txt"] = np.asarray(im).astype(np.float32) / 255
    yy, xx = np.mgrid[0:120, 0:160].astype(np.float32)
    img = np.zeros((120, 160, 3), np.float32)
    sky_ = np.clip(1 - np.abs(yy - 78) / 40, 0, 1)[..., None] * np.array([0.9, 0.1, 0.7])
    img += sky_
    below = yy > 80
    z = 400.0 / np.maximum(yy - 78, 1)
    gx = ((xx - 80) * z / 40) % 4 < 0.35
    gz = (z + t * 30) % 6 < 0.6
    img[below] += (np.array([1.0, 0.45, 0.05]) * (gx | gz)[below][:, None]).astype(np.float32)
    txt = _AIRSHIP_TXT["txt"]
    img = img * (1 - txt[..., None]) + txt[..., None] * np.array([0.2, 0.5, 1.0])
    img *= (0.85 + 0.15 * ((yy % 3) < 1))[..., None]
    return srgb_lin(np.clip(img, 0, 1))


SKIN_F_HAND = 389     # the night arrival's frame at the hand-off: the skin's scroll is on its frames
_MQ = {}


def marquee():
    """Every callout on one long line in the VGA font, 'TITLE - GROUP   *   ...' (as the night arrival's callouts.marquee)."""
    if "m" not in _MQ:
        from PIL import ImageFont
        text = "   *   ".join("%s - %s" % (a.upper(), b.upper()) for a, b in load_callouts()) + "   *   "
        f = ImageFont.truetype(os.path.join(HERE, "pcfonts", "Px437_IBM_VGA_8x16.ttf"), 16)
        w = int(ImageDraw.Draw(Image.new("L", (1, 1))).textlength(text, font=f)) + 8
        im = Image.new("L", (w, 16), 0)
        ImageDraw.Draw(im).text((0, 0), text, font=f, fill=255)
        _MQ["m"] = (np.array(im) > 127).astype(np.float32)
    return _MQ["m"]


def skin_light(q, t):
    """The outer wall's LED skin, exactly as the night arrival draws it (train_night.py, K_STADSKIN): two rows of
    the marquee scrolling opposite ways round the bowl, dots of light, the colour turning with the angle, pulsing on
    the beat (to 35% between)."""
    f = SKIN_F_HAND + (t - JOIN_T) * FPS
    ang = np.arctan2(q[:, 1], q[:, 0])
    s_ = ang * 125.0
    y = q[:, 2]
    mq = marquee()
    L = mq.shape[1]
    img_ = np.zeros(len(y))
    for (y0, y1, dirn) in ((11.0, 19.4, 1), (22.2, 30.6, -1)):
        row = (y1 - y) / (y1 - y0) * 16
        col = (s_ / ((y1 - y0) / 16) + dirn * f * 3.0) % L
        ok = (row >= 0) & (row < 16)
        r_, c_ = np.clip(row, 0, 15.99).astype(int), col.astype(int) % L
        dot = (np.abs(row % 1 - 0.5) < 0.35) & (np.abs(col % 1 - 0.5) < 0.35)
        img_ = np.maximum(img_, ok * mq[r_, c_] * (0.4 + 0.6 * dot))
    hue = ang * 3 + f * 0.05
    c = np.stack([0.5 + 0.5 * np.sin(hue), 0.5 + 0.5 * np.sin(hue + 2.1), 0.5 + 0.5 * np.sin(hue + 4.2)], 1)
    ph = ((f - SKIN_F_HAND) % 6) / 6.0
    pulse = 1.0 - 0.65 * (1 - math.exp(-4.0 * ph))
    return (c * (img_ * 4.0 * pulse)[:, None]).astype(np.float32)


def frame_rgb(sc, t, w=W, h=H, stats=None):
    if t >= T_CUT - 1e-6:          # the music stops: a hard cut to black
        return np.zeros((h, w, 3), np.uint8)
    eye, tgt = camera(t)
    img = render(sc, eye, tgt, t, w, h, stats)
    return (img * 255 + 0.5).astype(np.uint8)


_SC = None


def _worker_frame(args):
    global _SC
    if _SC is None:
        _SC = Scene()
    f, path = args
    rgb = frame_rgb(_SC, f / FPS)
    rgb.tofile(path)
    return f


def film(out, small=False, frames=None, procs=6, start=0.0):
    import multiprocessing as mp
    n = int(T_END * FPS)
    a, b = frames if frames else (int(round(start * FPS)), n)
    tmp = WORK + f"frames_{os.environ.get('TAG', 'v1')}/"
    os.makedirs(tmp, exist_ok=True)
    jobs = [(f, f"{tmp}{f:04d}.rgb") for f in range(a, b)]
    todo = [j for j in jobs if not os.path.exists(j[1])]
    with mp.get_context("spawn").Pool(procs) as pool:
        for f in pool.imap_unordered(_worker_frame, todo):
            print("frame", f, file=sys.stderr, flush=True)
    encode(out, [j[1] for j in jobs], small)


def encode(out, paths, small):
    scale = ["-vf", "scale=1536:768:flags=neighbor"] if not small else ["-vf", "scale=768:384:flags=area"]
    cmd = ["ffmpeg", "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r",
           str(FPS), "-i", "-", *scale, "-c:v", "libx264", "-profile:v", "main", "-level", "4.0", "-bf", "0", "-g",
           "12", "-pix_fmt", "yuv420p", "-crf", "20" if not small else "24", "-movflags", "+faststart", out]
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE)
    for path in paths:
        p.stdin.write(open(path, "rb").read())
    p.stdin.close()
    p.wait()


def sheet(out, times=(0.0, 2.0, 3.6, 4.6, 6.0, 9.5, 13.0, 16.5)):
    sc = Scene()
    ims = []
    for t in times:
        st = {}
        rgb = frame_rgb(sc, t, stats=st)
        print(f"t={t}: {st}", file=sys.stderr)
        ims.append(Image.fromarray(rgb))
    cols = 2
    S = Image.new("RGB", (W // 2 * cols, H // 2 * ((len(ims) + 1) // cols)))
    for i, im in enumerate(ims):
        S.paste(im.resize((W // 2, H // 2), Image.LANCZOS), ((i % cols) * W // 2, (i // cols) * H // 2))
    S.save(out)
    # and the across-the-room check: every still at 384x192
    S2 = Image.new("RGB", (384 * 4, 192 * ((len(ims) + 3) // 4)))
    for i, im in enumerate(ims):
        S2.paste(im.resize((384, 192), Image.BOX), ((i % 4) * 384, (i // 4) * 192))
    S2.save(out.replace(".png", "_384.png"))


PHOTOS = [   # cut-out, height in metres, top above the floor (metres); all public domain or CC0 (docs/design)
    ("Person_s_back_Glasgow_1.png", 0.55, 1.80), ("Wander_Unsplash_1.png", 0.70, 1.82),
    ("Woman_facing_Painted_1.png", 0.85, 1.70), ("Watching_football_in_jersey_1.png", 0.55, 1.62),
    ("Spectator_in_bobble_1.png", 0.85, 1.85), ("A_man_posing_1.png", 0.85, 2.05),
    ("People_watching_Shadowverse_1.png", 1.0, 1.78), ("People_watching_Shadowverse_2.png", 1.0, 1.75),
    ("Female_from_behind_1.png", 1.68, 1.68),
]


def load_photos():
    imgs, rims, size = [], [], []
    for f, hm, top in PHOTOS:
        im = Image.open(WORK + "cutouts/" + f).convert("RGBA")
        im.thumbnail((720, 720))
        a = np.asarray(im).astype(np.float32) / 255
        al = a[..., 3]
        # the rim: the outline's inside edge (the alpha less its erosion), as the screen's light catches it
        er = np.asarray(Image.fromarray((al * 255).astype(np.uint8)).filter(
            __import__("PIL.ImageFilter", fromlist=["MinFilter"]).MinFilter(5))).astype(np.float32) / 255
        imgs.append(a)
        rims.append(np.clip(al - er, 0, 1))
        size.append((hm, top))
    return dict(imgs=imgs, rims=rims, size=size)


def compare(out, t=10.0):
    """The same frame twice: the near crowd as the posed models, then as photo cut-outs; and both at 384x192."""
    sc = Scene()
    eye, tgt = camera(t)
    a = (np.clip(render(sc, eye, tgt, t), 0, 1) * 255).astype(np.uint8)
    b = (np.clip(render(sc, eye, tgt, t, photos=load_photos()), 0, 1) * 255).astype(np.uint8)
    top = np.concatenate([a, b], 1)
    small = np.concatenate([np.asarray(Image.fromarray(x).resize((384, 192), Image.BOX)) for x in (a, b)], 1)
    S = Image.new("RGB", (W * 2, H + 192), (20, 20, 20))
    S.paste(Image.fromarray(top), (0, 0))
    S.paste(Image.fromarray(small), (0, H))
    S.save(out)


def main():
    cmd = sys.argv[1]
    if cmd == "prep":
        prep()
    elif cmd == "still":
        sc = Scene()
        st = {}
        Image.fromarray(frame_rgb(sc, float(sys.argv[2]), stats=st)).save(sys.argv[3])
        print(st, file=sys.stderr)
    elif cmd == "sheet":
        sheet(sys.argv[2])
    elif cmd == "compare":
        compare(sys.argv[2], float(sys.argv[3]) if len(sys.argv) > 3 else 10.0)
    elif cmd == "film":
        small = "--small" in sys.argv
        fr = None
        procs = 6
        start = 0.0
        for i, a in enumerate(sys.argv):
            if a == "--start":                 # --start 3 (JOIN_T): the film as joined to the night arrival
                start = float(sys.argv[i + 1])
            if a == "--frames":
                fr = tuple(int(x) for x in sys.argv[i + 1].split(":"))
            if a == "--procs":
                procs = int(sys.argv[i + 1])
        film(sys.argv[2], small, fr, procs, start)
    elif cmd == "encode":
        n = int(T_END * FPS)
        f0 = int(round(float(sys.argv[sys.argv.index("--start") + 1]) * FPS)) if "--start" in sys.argv else 0
        encode(sys.argv[2], [f"{WORK}frames_{os.environ.get('TAG', 'v1')}/{f:04d}.rgb" for f in range(f0, n)],
               "--small" in sys.argv)


if __name__ == "__main__":
    main()
