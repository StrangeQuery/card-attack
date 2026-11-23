"""The stadium party's data: the scene of the prototype (prototypes/stadium.py) in whole numbers, and its pictures.

    python segments/stadium/data/make_data.py          # writes scene.bin and pictures.bin here (not kept in git)

Everything here is an asset or is placed once: the stadium's triangles (Blender, audience/stadium_build.py), the
dancers' posed meshes and their impostors (Blender, audience/pose_people.py; the impostors drawn from the poses
here, as the prototype's make_atlas does), the airship (models/blimp.obj, the night arrival's), where the 65,536 people,
the flags, the balloons, the confetti, the city's lights and the stars are (the prototype's own random draws), the
lights' places and colours; the pictures (the demos' footage, the type pre-rendered in the prototype's fonts, the
national flags drawn from their designs, the CARDIAC; the 8 demos we may show as footage, the 55 parodies'
stand-ins as the numbers CARDIAC works them out from). Nothing that moves is worked out here: the machines do that
each frame.

Units: places in centimetres (the prototype's metres x 100), normals in hundredths, colours in ten-thousandths of
the prototype's linear light, angles in millionths of a turn. Pictures stay as their sRGB bytes; the table srgb_lin
turns a byte into linear light.
"""
import math
import os
import struct
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "prototypes"))
os.environ.setdefault("STADIUM_END", "47.25")
import stadium as P  # noqa: E402

OUT = os.path.join(HERE, "scene.bin")
PICS = os.path.join(HERE, "pictures.bin")
ARR, PIC = [], []



def camera_keys():
    """The camera's keys (segments/stadium/clock.rs KEYS): times in twelve-thousandths of a second, eye, look (cm)."""
    import re
    text = open(os.path.join(HERE, "..", "clock.rs")).read()
    body = text[text.index("pub const KEYS"):]
    body = body[:body.index("];")]
    return [(int(t), [int(a), int(b), int(c)]) for t, a, b, c in
            re.findall(r"\((\d+), \[(-?\d+), (-?\d+), (-?\d+)\]", body)]


def camera_path(step=250):
    """The eye along the keys (metres), as clock.rs's camera() draws it: Catmull-Rom on the keys' own times."""
    keys = camera_keys()
    ts, ps = [k[0] for k in keys], [np.array(k[1], float) for k in keys]
    out = []
    for i in range(len(keys) - 1):
        k0, k1, k2, k3 = max(i - 1, 0), i, i + 1, min(i + 2, len(keys) - 1)
        d21, d20, d31 = ts[k2] - ts[k1], max(ts[k2] - ts[k0], 1), max(ts[k3] - ts[k1], 1)
        m1, m2 = (ps[k2] - ps[k0]) * d21 / d20, (ps[k3] - ps[k1]) * d21 / d31
        for t in range(ts[k1], ts[k2], step):
            u = (t - ts[k1]) / d21
            h = (2 * u**3 - 3 * u**2 + 1, u**3 - 2 * u**2 + u, -2 * u**3 + 3 * u**2, u**3 - u**2)
            out.append((h[0] * ps[k1] + h[1] * m1 + h[2] * ps[k2] + h[3] * m2) / 100)
    return np.array(out)


AISLES = (-6.0, 30.0)       # the two aisles up the pitch (the first almost under the airship), their
                            # middles' x (m); 4 m wide


def aisles(pos, on_pitch, half=2.0):
    """The two aisles up the pitch, 4 m wide, the whole length (the camera runs up the left one
    just over the heads and pulls back down the right): anyone on the pitch within 2 m of an aisle's middle is
    moved out to its edge on their own side, 0 to 45 cm further at random; nobody is taken away (65,536 stay)."""
    pos = pos.copy()
    rng = np.random.default_rng(1602)
    for xa in AISLES:
        d = pos[:, 0] - xa
        hit = on_pitch & (np.abs(d) < half)
        side = np.where(d[hit] < 0, -1.0, 1.0)
        pos[hit, 0] = xa + side * (half + 0.45 * rng.random(hit.sum()))
        print(f"aisle at x {xa:g} m: {hit.sum()} people moved to its edges", file=sys.stderr)
    return pos


def aisle(pos, on_pitch, reach=0.8, low=1.9):
    """People on the pitch moved out of the camera's way wherever else it flies low (in the aisles nobody is that
    close): anyone within `reach` (m) across of the eye's path below `low` m is pushed straight
    out from it to the aisle's edge, 0 to 30 cm further at random; nobody is taken away (65,536 stay)."""
    path = camera_path()
    path = path[path[:, 2] < low][:, :2]
    if len(path) == 0:
        return pos
    pos = pos.copy()
    rng = np.random.default_rng(1601)
    idx = np.nonzero(on_pitch)[0]
    xy = pos[idx, :2]
    best_d = np.full(len(idx), 1e9)
    best_q = np.zeros((len(idx), 2))
    for a in range(0, len(path), 400):
        q = path[a:a + 400]
        d = np.hypot(xy[:, None, 0] - q[None, :, 0], xy[:, None, 1] - q[None, :, 1])
        j = d.argmin(1)
        dd = d[np.arange(len(idx)), j]
        better = dd < best_d
        best_d[better] = dd[better]
        best_q[better] = q[j[better]]
    hit = best_d < reach
    away = xy[hit] - best_q[hit]
    n = np.hypot(away[:, 0], away[:, 1])
    away = np.where(n[:, None] > 1e-6, away / np.maximum(n, 1e-6)[:, None], np.array([1.0, 0.0]))
    pos[idx[hit], :2] = best_q[hit] + away * (reach + 0.3 * rng.random(hit.sum()))[:, None]
    print(f"aisle: {hit.sum()} people moved out of the camera's way", file=sys.stderr)
    return pos

def put(name, a, dest=None):
    a = np.asarray(a)
    if a.dtype.kind == "f":
        a = np.round(a)
    a = np.ascontiguousarray(a.astype("<i4"))
    assert np.abs(a).max(initial=0) < 2 ** 31
    (ARR if dest is None else dest).append((name, 0, a))


def pic(name, a):
    a = np.ascontiguousarray(np.asarray(a).astype(np.uint8))
    PIC.append((name, 1, a))


def put8(name, a):
    """Small signed numbers (an atlas: -99..99), a byte each."""
    a = np.asarray(a)
    assert np.abs(a).max() < 128
    PIC.append((name, 2, np.ascontiguousarray(a.astype(np.int8))))


def write(path, arrs):
    with open(path, "wb") as f:
        for name, kind, a in arrs:
            nb = name.encode()
            f.write(struct.pack("<I", len(nb)) + nb + struct.pack("<II", kind, a.ndim))
            f.write(struct.pack("<%dI" % a.ndim, *a.shape))
            f.write(a.tobytes())
    print(path, os.path.getsize(path) // 1000, "kB", file=sys.stderr)


def cm(x):
    return np.round(np.asarray(x, float) * 100).astype(np.int64)


def hund(n):
    n = np.asarray(n, float)
    n = n / (np.linalg.norm(n, axis=-1, keepdims=True) + 1e-12)
    return np.round(n * 99).astype(np.int64)        # 99 is a unit: a part squared and summed stays under 10,000


def lin4(c):
    return np.round(np.asarray(c, float) * 10000).astype(np.int64)


def turn(a):
    return np.round(np.asarray(a, float) / (2 * math.pi) * 1e6).astype(np.int64) % 1000000


def to_byte(lin):
    """Linear light 0..1 back to the sRGB byte the prototype made it from (its srgb_lin is a power of 2.2)."""
    return np.clip(np.round(np.clip(lin, 0, 1) ** (1 / 2.2) * 255), 0, 255).astype(np.uint8)


def subdivide(p, n, m, longest):
    """Triangles split at their longest edge's middle until no edge is longer than `longest` (metres)."""
    out_p, out_n, out_m = [], [], []
    stack = list(zip(p, n, m))
    while stack:
        tp, tn, tm = stack.pop()
        e = [np.linalg.norm(tp[(i + 1) % 3] - tp[i]) for i in range(3)]
        i = int(np.argmax(e))
        if e[i] <= longest:
            out_p.append(tp); out_n.append(tn); out_m.append(tm)
            continue
        j, k = (i + 1) % 3, (i + 2) % 3
        mp, mn = (tp[i] + tp[j]) / 2, (tn[i] + tn[j]) / 2
        stack.append((np.array([tp[i], mp, tp[k]]), np.array([tn[i], mn, tn[k]]), tm))
        stack.append((np.array([mp, tp[j], tp[k]]), np.array([mn, tn[j], tn[k]]), tm))
    return np.array(out_p, np.float32), np.array(out_n, np.float32), np.array(out_m)


def weld(p, n):
    """Triangle corners (m, 3, 3) and their normals into shared vertices: (vertices, normals, triangles)."""
    key = np.concatenate([cm(p).reshape(-1, 3), hund(n).reshape(-1, 3)], 1)
    u, inv = np.unique(key, axis=0, return_inverse=True)
    return u[:, :3], u[:, 3:], inv.reshape(-1, 3)


def main():
    sc = P.Scene()
    info = sc.info
    # ---- the stadium -----------------------------------------------------------------------------------------
    # (the ground's and the pitch's big triangles cut down to edges of 150 m or less: a corner made where an edge is
    # cut at the near plane is placed to a thousandth of the edge)
    sp, sn, sm = subdivide(sc.sp, sc.sn, np.asarray(sc.sm), 150.0)
    sc.sp, sc.sn, sc.sm = sp, sn, sm
    v, vn, t = weld(sc.sp, sc.sn)
    put("st_v", v)
    put("st_vn", vn)
    put("st_t", t)
    put("st_mat", sc.sm)
    mats = info["materials"]
    kind = [2 if m["name"] == "facade" else (1 if m["kind"] == "emit" else 0) for m in mats]
    put("mat_kind", kind)
    put("mat_alb", lin4([m["rgb"] for m in mats]))
    put("mat_emit", lin4([np.array(m["rgb"]) * (m.get("emit", 0.0) if m["kind"] == "emit" else 0.0) for m in mats]))
    # the facade (the LED skin) as triangles of its own: their corners' angles round the bowl (millionths of a turn,
    # unwrapped across the seam at the back)
    fac = np.array(sc.sm) == sc.m_facade
    fp = sc.sp[fac]
    ang = np.arctan2(fp[..., 1], fp[..., 0]) / (2 * math.pi) * 1e6
    ang = np.where((ang.max(1, keepdims=True) - ang.min(1, keepdims=True) > 5e5) & (ang < 0), ang + 1e6, ang)
    put("sk_v", cm(fp.reshape(-1, 3)))
    put("sk_ang", np.round(ang))
    # the skin's places: the angle round the bowl (s = 125 angle, in thousandths) and the height, at each corner of
    # a facade triangle (the angle unwrapped across the seam at the back)
    put("bowl", [info["bowl"]["A0"] * 100, info["bowl"]["B0"] * 100, info["bowl"]["rows"], info["bowl"]["run"] * 100,
                 info["bowl"]["rise"] * 100, info["bowl"]["z0"] * 100, info["bowl"]["open_y"] * 100])

    # ---- the light probes: where the light is worked out each frame; everything lit takes its nearest's ----------
    # (a 4 m grid over the pitch, rings round every fourth row of the tiers every 6 m, the stadium's places in 12 m
    # cells; the records' names end in "_hd")
    b = info["bowl"]
    vm = v / 100.0
    pos = np.stack([sc.cx, sc.cy, sc.cz], 1)
    pos = aisles(pos, sc.ckind == 0)
    pos = aisle(pos, sc.ckind == 0)

    def make_probes(grid, ring_rows, ring_gap, cell, sfx):
        gx, gy = np.meshgrid(np.arange(-75.0, 76.0, grid), np.arange(-55.0, 36.0, grid))
        pitch = np.stack([gx.ravel(), gy.ravel(), np.full(gx.size, 1.0)], 1)
        rings = []
        for k in range(0, b["rows"], ring_rows):
            rx, ry = b["A0"] + (k + 1.0) * b["run"], b["B0"] + (k + 1.0) * b["run"]
            per = 2 * math.pi * math.sqrt((rx * rx + ry * ry) / 2)
            n = int(per / ring_gap)
            am = np.arange(n) * 2 * math.pi / n
            rings.append(np.stack([rx * np.cos(am), ry * np.sin(am), np.full(n, b["z0"] + (k + 0.5) * b["rise"] + 1.0)], 1))
        rings = np.concatenate(rings)
        vox = np.unique(np.floor(vm / cell), axis=0)
        occupied = []
        for c in vox:
            sel = np.all(np.floor(vm / cell) == c, axis=1)
            occupied.append(vm[sel].mean(0))
        probes = np.concatenate([pitch, rings, np.array(occupied)])
        put("probes" + sfx, cm(probes))
        print("probes" + sfx + ":", len(probes), file=sys.stderr)

        def nearest_probe(pts, among=None):
            pr = probes if among is None else probes[among]
            out = np.zeros(len(pts), np.int64)
            for s in range(0, len(pts), 4096):
                d = ((pts[s:s + 4096, None, :] - pr[None, :, :]) ** 2).sum(-1)
                out[s:s + 4096] = np.argmin(d, 1)
            return out if among is None else np.asarray(among)[out]

        put("st_vprobe" + sfx, nearest_probe(vm))
        put("cprobe" + sfx, nearest_probe(pos + np.array([0, 0, 1.0]), np.arange(len(pitch) + len(rings))))

    make_probes(2.0, 2, 3.0, 6.0, "")
    make_probes(4.0, 4, 6.0, 12.0, "_hd")

    # ---- the crowd ---------------------------------------------------------------------------------------------
    put("cpos", cm(pos))
    put("cface_cs", np.round(np.stack([np.cos(sc.cface - math.pi / 2), np.sin(sc.cface - math.pi / 2)], 1) * 1000))
    put("cparty", sc.cparty.astype(int))
    put("cmodel", sc.cmodel)
    put("cmove", sc.cmove)
    put("cphase", sc.cphase)
    put("cscale", np.round(sc.cscale * 1000))
    # the clothes: dark solid colours, no prints: navy, maroon, forest, purple, charcoal, teal,
    # black, each person one at random
    hue = np.array([[0.45, 0.6, 1.95], [1.9, 0.55, 0.55], [0.55, 1.6, 0.85], [1.35, 0.5, 1.75], [1.0, 1.0, 1.05],
                    [0.4, 1.3, 1.3], [0.55, 0.55, 0.6]])
    tint = hue[np.random.default_rng(1603).integers(0, len(hue), len(sc.ctone))]
    put("ctone", lin4(0.025 * sc.ctone[:, None] * tint))
    put("cglow", np.round(0.025 * sc.ctone[:, None] * tint * 0.004 * 10000))
    put("cshirt", np.full(len(sc.cshirt), -1))
    put("ckind", sc.ckind)
    put("cang", turn(np.arctan2(sc.cy / 58.0, sc.cx / 78.0)) // 2)      # two-millionths of a turn
    put("cphone", turn(np.arange(len(sc.cx)) * 1.7))                  # the phone's flicker's phase (ids 1.7 rad)
    # who holds the flags and placards (they are the point; concentrated by the aisles): along both
    # aisles, 2.3 to 6 m from their middles; a few round the circle between them (as many as the prototype's)
    on = np.nonzero(sc.ckind == 0)[0]
    da = np.min(np.abs(pos[on, 0:1] - np.array(AISLES)[None]), 1)
    yy = pos[on, 1]
    frng = np.random.default_rng(1604)
    by_aisle = frng.choice(on[(da > 2.3) & (da < 6.0) & (yy > -46) & (yy < 18)], len(sc.flag_pid) - 10, replace=False)
    left = np.setdiff1d(on, by_aisle)
    dl = np.min(np.abs(pos[left, 0:1] - np.array(AISLES)[None]), 1)
    by_circle = frng.choice(left[(pos[left, 1] > 2) & (pos[left, 1] < 24) & (np.abs(pos[left, 0]) < 16) & (dl > 2.3)
                                 & (np.abs(pos[left, 1] - 13) > 2.5)], 10, replace=False)
    put("flag_pid", np.r_[by_aisle, by_circle])
    put("flag_tex", sc.flag_tex)
    put("flag_ph", turn(sc.flag_ph))

    # ---- the screens: the giant screen, the LED walls, the banners, as grids of small quads (their pictures by
    # the place on the quad, thousandths of a texel) -------------------------------------------------------------
    qv, qt, quv, qk, qa = [], [], [], [], []

    def grid(c, wid, hei, rgt, upv, nx, ny, tw, th, k, aux):
        c, rgt, upv = np.array(c, float), np.array(rgt, float), np.array(upv, float)
        base = len(qv)
        for j in range(ny + 1):
            for i in range(nx + 1):
                qv.append(c + rgt * wid * (i / nx - 0.5) + upv * hei * (j / ny - 0.5))
        for j in range(ny):
            for i in range(nx):
                a_, b_, c_, d_ = base + j * (nx + 1) + i, base + j * (nx + 1) + i + 1, base + (j + 1) * (nx + 1) + i + 1, \
                    base + (j + 1) * (nx + 1) + i
                uv = lambda q: (round((q - base) % (nx + 1) / nx * tw * 1000), round((1 - (q - base) // (nx + 1) / ny) * th * 1000))
                for tri in ((a_, b_, c_), (a_, c_, d_)):
                    qt.append(tri)
                    quv.append([uv(q) for q in tri])
                    qk.append(k)
                    qa.append(aux)

    # (the giant screen and the LED walls stand 30 cm further out from the wall and the frame behind them than the
    # prototype's 5 cm: the depths' whole numbers can't tell 5 cm apart at 60 m)
    s = info["screen"]
    sc_c = np.array(s["centre"]) - [0, 0.3, 0]
    grid(sc_c, s["width"], s["height"], (1, 0, 0), (0, 0, 1), 24, 12, 384, 192, 0, 0)
    for i, lw in enumerate(info["led_walls"]):
        grid([lw["centre"][0], lw["centre"][1] - 0.35, lw["centre"][2]], lw["width"], lw["height"], (1, 0, 0), (0, 0, 1),
             4, 8, 120, 220, 1, i)
    for c_, n_, k_ in sc.banners:
        grid(c_, 12.0, 7.5, (-n_[1], n_[0], 0.0), (0, 0, 1), 2, 2, 160, 100, 2, k_)
    put("q_v", cm(qv))
    put("q_t", qt)
    put("q_uv", quv)
    put("q_kind", qk)
    put("q_aux", qa)
    put("screen", [cm(s["centre"])[0], cm(s["centre"])[1], cm(s["centre"])[2], s["width"] * 100, s["height"] * 100])
    # the screen's 18 parts as area lights (3 rows of 6), their middles
    c0 = np.array(s["centre"])
    put("screen_parts", cm([c0 + np.array([(ix + 0.5) / 6 - 0.5, 0, 0.5 - (iy + 0.5) / 3]) * [s["width"], 0, s["height"]]
                            for iy in range(3) for ix in range(6)]))
    put("led_walls", [[*cm(lw["centre"]), lw["width"] * 100, lw["height"] * 100] for lw in info["led_walls"]])

    # ---- the lights ---------------------------------------------------------------------------------------------
    put("spots", [[*cm(l["pos"]), *lin4(l["rgb"]), turn(l["phase"]), l["cone"]] for l in sc.spots])
    se = []
    for l in sc.search:
        mode = 2 if l.get("sweep") else (1 if l.get("aim") else 0)
        se.append([*cm(l["pos"]), *lin4(l["rgb"]), round(l["power"] * 1000), turn(l["phase"]), mode,
                   round(l["phase"] * 1000)])
    put("search", se)
    put("lasers", [[*cm(l["pos"]), *lin4(l["rgb"]), l["n"], l["spread"]] for l in sc.lasers])
    put("pyro", cm(info["pyro"]))

    # ---- the people's poses: per model the shared triangles; per move and phase the corners (centimetres, facing
    # +Y, feet at 0), their normals, the lift, the hands -------------------------------------------------------
    people = sc.people
    for mi, m in enumerate(P.MODELS):
        t_ = people.t[m]
        v0 = people.v[m, "jump"][0]
        _, weld_i, inv = np.unique(np.round(v0, 4), axis=0, return_index=True, return_inverse=True)
        inv = inv.ravel()
        put(f"pm{mi}_t", inv[t_])
        vs = np.stack([np.stack([people.v[m, mv][p][weld_i] for p in range(P.NPHASE)]) for mv in P.MOVES])
        ns = np.stack([np.stack([people.n[m, mv][p][weld_i] for p in range(P.NPHASE)]) for mv in P.MOVES])
        put(f"pm{mi}_v", cm(vs))
        put(f"pm{mi}_n", hund(ns))
        # each corner's normal along its place (for the rim: hundredths times centimetres), and the back print's place
        # on it: across and up (thousandths of the print), how far in front of the back (millimetres)
        put(f"pm{mi}_nv", (hund(ns) * cm(vs)).sum(-1))
        put(f"pm{mi}_su", np.round((vs[..., 0] + 0.2) / 0.4 * 1000))
        put(f"pm{mi}_sv", np.round((vs[..., 2] - 1.0) / 0.42 * 1000))
        put(f"pm{mi}_sy", np.round(vs[..., 1] * 1000))
        put(f"pm{mi}_lift", cm(np.stack([people.lift[m, mv] for mv in P.MOVES])))
    # the impostors: the prototype's atlas (56 x 96), and one three times as fine for the nearer, drawn the same way
    put("atlas_hand", cm(sc.atlas_hand))
    put8("atlas", atlas_ints(sc.atlas))
    put8("atlas_hi", atlas_ints(make_atlas(people, 3)))

    # ---- the flags, balloons, confetti, city, stars -----------------------------------------------------------
    put("bal", np.stack([cm(sc.bal_x), cm(sc.bal_y), np.round(sc.bal_t0 * 1000), cm(sc.bal_v), cm(sc.bal_r),
                         turn(sc.bal_ph)], 1))
    put("bal_c", lin4(sc.bal_c))
    put("conf", np.stack([cm(sc.conf_p[:, 0]), cm(sc.conf_p[:, 1]), np.round(sc.conf_p[:, 2] * 5500),
                          turn(sc.conf_ph), np.round(sc.conf_fall * 1000)], 1))
    put("conf_c", lin4(sc.conf_c))
    put("city", cm(sc.city))
    put("city_c", lin4(sc.city_rgb))
    put("stars", np.round(sc.stars * 999))
    # Dope's stars: crisp white points, about one per 650 pixels of open sky at the screen's middle (1 / 650
    # of the solid angle of 1,000 x 1,000 pixels at f = 1000: about 1,540 a steradian; a quarter more for the screen's edges, where a pixel is less sky),
    # bright 140..255 as bytes after the tone curve (as linear light: the curve undone), a quarter of them 2 x 2
    rng = np.random.default_rng(1995)
    nd = int(1540 * 4 * math.pi * 1.25)
    d = rng.normal(size=(nd, 3))
    d /= np.linalg.norm(d, axis=1, keepdims=True)
    d = d[d[:, 2] > -0.1]
    b = rng.uniform(140, 255, len(d)) / 255.0
    lin = -np.log(1 - np.minimum(b ** 2.2, 0.995)) / 1.25
    put("dstars", np.round(d * 999))
    put("dstar_l", np.round(lin * 10000))
    put("dstar_big", (rng.random(len(d)) < 0.25).astype(int))
    put("star_b", lin4(sc.star_b))

    # ---- the airship ------------------------------------------------------------------------------------------
    bl = sc.blimp
    bp, bn, bm = bl["P"], bl["N"], np.asarray(bl["M"])
    # the screen's triangles cut in 16 each (its picture's place is carried straight across a triangle)
    scr = bm == 3
    sub_p, sub_n = [], []
    for tri, nrm in zip(bp[scr], bn[scr]):
        k = 4
        for i in range(k):
            for j in range(k - i):
                w = lambda a, b: tri[0] + (tri[1] - tri[0]) * a / k + (tri[2] - tri[0]) * b / k
                sub_p.append([w(i, j), w(i + 1, j), w(i, j + 1)])
                sub_n.append(nrm)
                if i + j < k - 1:
                    sub_p.append([w(i + 1, j), w(i + 1, j + 1), w(i, j + 1)])
                    sub_n.append(nrm)
    bp = np.concatenate([bp[~scr], np.array(sub_p)])
    bn = np.concatenate([bn[~scr], np.array(sub_n)])
    bm = np.concatenate([bm[~scr], np.full(len(sub_p), 3)])
    bv, bvn, btri = weld(bp, bn)
    # each corner's place on the screen's picture (thousandths across and up: the prototype's a_, b_)
    bs = P.BLIMP_SCREEN
    upv = np.array([0.0, -math.sin(bs["tilt"]), math.cos(bs["tilt"])])
    q = bv / 100.0 - bs["c"]
    put("bl_ab", np.stack([np.round(np.clip(q[:, 0] / bs["w"] + 0.5, 0, 1) * 1000), np.round(np.clip((q @ upv) / bs["h"] + 0.5, 0, 1) * 1000)], 1))
    # the airship's light probes: the middles of its corners in 4 m cells (in its own frame: they ride with it); the
    # party version's in 8 m cells
    for size, sfx in ((400.0, ""), (800.0, "_hd")):
        cells, inv = np.unique(np.floor(bv / size), axis=0, return_inverse=True)
        inv = inv.ravel()
        bprobe = np.array([bv[inv == k].mean(0) for k in range(len(cells))])
        put("bl_probes" + sfx, np.round(bprobe))
        put("bl_vprobe" + sfx, inv)
        print("airship probes" + sfx + ":", len(bprobe), file=sys.stderr)
    put("bl_v", bv)
    put("bl_vn", bvn)
    put("bl_t", btri)
    put("bl_mat", bm)
    put("bl_lights", cm(bl["lights"]))
    put("bl_beacons", cm(bl["beacons"]))
    put("blimp", [*cm(P.BLIMP_AT), round(P.BLIMP_DRIFT[0] * 100), P.BLIMP_SCALE * 1000])
    bs = P.BLIMP_SCREEN
    put("bl_screen", [*cm(bs["c"]), bs["w"] * 100, bs["h"] * 100, turn(bs["tilt"])])

    # ---- the pictures -------------------------------------------------------------------------------------------
    demos = sc.demos
    clips = P.clips_of(demos)
    assert len(clips) == 8 and all(len(d["frames"]) == 60 for d in clips), "the 8 clips' 60 frames"
    pic("demos", np.stack([d["frames"] for d in clips]))
    # each callout: its clip (0..7) or -1; the parodies' stand-in (kind 0..4: plasma, tunnel, stars, rotozoom, rings;
    # -1 for a clip) and hue; the starfields' stars; the stand-ins' printed numbers (P.standin_tables)
    ci = {id(d): k for k, d in enumerate(clips)}
    put("demo_clip", [ci.get(id(d), -1) for d in demos])
    put("standin", [[d["kind"], d["hue"]] for d in demos])
    put("sd_stars", np.stack([d["stars"] for d in demos]))
    sd = P.standin_tables()
    for k in ("pl", "tu", "xy", "sin64", "half", "tun", "par", "rot", "rsq", "ring", "sq", "pal", "vb"):
        put("sd_" + k, sd[k])
    pic("captions", np.stack([to_byte(d["caption"]) for d in demos]))
    pic("vcaptions", np.stack([to_byte(P.vertical_caption(d["title"], d["group"])) for d in demos]))
    pic("flags", np.stack([to_byte(im) for im in sc.flag_img]))
    pic("shirts", np.stack([np.concatenate([to_byte(im[..., :3]), np.round(im[..., 3:] * 255).astype(np.uint8)], -1)
                            for im in sc.shirt_img]))
    pic("thanks_f", to_byte(P.thanks_fingerman(0)))
    pic("thanks_h", to_byte(P.thanks_hagelbarger(0)))
    pic("cardiac", to_byte(P.cardiac_picture() / 1.2))
    P.airship_picture(sc, 2.6)
    pic("airship_txt", np.round(P._AIRSHIP_TXT["txt"] * 255))
    a_, b_, z_ = airship_grid()
    pic("airship_a", a_)
    pic("airship_b", b_)
    put("airship_z", z_)
    pic("marquee", P.marquee())
    write(OUT, ARR)
    write(PICS, PIC)


def airship_grid():
    """The airship's DEMOSPLASH 0x10 picture (the prototype's airship_picture) in its two states, a row's grid line
    off and on (the moving part, (z + 30 t) mod 6 < 0.6, is the machines'), and each row's z (thousandths; -1 above
    the horizon)."""
    txt = P._AIRSHIP_TXT["txt"]
    yy, xx = np.mgrid[0:120, 0:160].astype(np.float32)
    out = []
    for lit in (False, True):
        img = np.zeros((120, 160, 3), np.float32)
        img += np.clip(1 - np.abs(yy - 78) / 40, 0, 1)[..., None] * np.array([0.9, 0.1, 0.7])
        below = yy > 80
        z = 400.0 / np.maximum(yy - 78, 1)
        gx = ((xx - 80) * z / 40) % 4 < 0.35
        g = (gx | lit)
        img[below] += (np.array([1.0, 0.45, 0.05]) * g[below][:, None]).astype(np.float32)
        img = img * (1 - txt[..., None]) + txt[..., None] * np.array([0.2, 0.5, 1.0])
        img *= (0.85 + 0.15 * ((yy % 3) < 1))[..., None]
        out.append(np.round(np.clip(img, 0, 1) * 255))
    zr = np.where(np.arange(120) > 80, np.round(400.0 / np.maximum(np.arange(120) - 78, 1) * 1000), -1)
    return out[0], out[1], zr


def atlas_ints(at):
    """An atlas as whole numbers: whether each texel is covered (the prototype's cut at a half), and its normal in
    hundredths (across, up, toward the camera)."""
    at = np.asarray(at, np.float32)
    a = (at[..., 0] >= 0.5).astype(np.int64)
    n = at[..., 1:4]
    n = np.where(a[..., None] > 0, n / (np.linalg.norm(n, axis=-1, keepdims=True) + 1e-9), 0)
    return np.concatenate([a[..., None], np.round(n * 99).astype(np.int64)], -1)


def make_atlas(people, k):
    """The prototype's impostors at k times its size (56 x 96), from the same poses, from behind and 14 degrees
    above."""
    w, h = P.SPR_W * k, P.SPR_H * k
    n = len(P.MODELS) * len(P.MOVES) * P.NPHASE
    atlas = np.zeros((n, h, w, 4), np.float32)
    ce, se = math.cos(P.SPR_ELEV), math.sin(P.SPR_ELEV)
    for mi, m in enumerate(P.MODELS):
        for vi, mv in enumerate(P.MOVES):
            for p in range(P.NPHASE):
                q = (mi * len(P.MOVES) + vi) * P.NPHASE + p
                v, nn, t = people.v[m, mv][p], people.n[m, mv][p], people.t[m]
                up = v[:, 2] * ce + v[:, 1] * se
                depth = v[:, 1] * ce - v[:, 2] * se
                sx = (v[:, 0] / P.SPR_MW + 0.5) * w
                sy = (1 - (up + 0.1) / P.SPR_MH) * h
                tri, bary, _ = P.raster_ortho(sx[t], sy[t], depth[t], w, h, ss=2)
                cov = tri >= 0
                nrm = (nn[t[tri[cov]]] * bary[cov][:, :, None]).sum(1)
                ns = np.stack([nrm[:, 0], nrm[:, 2] * ce + nrm[:, 1] * se, -(nrm[:, 1] * ce - nrm[:, 2] * se)], 1)
                ns /= np.linalg.norm(ns, axis=1, keepdims=True) + 1e-9
                img = np.zeros((h * 2, w * 2, 4))
                img.reshape(-1, 4)[cov.ravel()] = np.c_[np.ones(cov.sum()), ns]
                img = img.reshape(h, 2, w, 2, 4).mean((1, 3))
                a = img[..., 0:1]
                img[..., 1:] = np.where(a > 0, img[..., 1:] / np.maximum(a, 1e-6), 0)
                atlas[q] = img
        print("atlas", m, file=sys.stderr)
    return atlas


if __name__ == "__main__":
    main()
