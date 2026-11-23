"""The bullet train's data, from the prototype (prototypes/train.py, which builds the scene from the Blender models
of prototypes/models/build_models.py in $CARD_ATTACK_WORK/models, or TRAIN_MODELS): the world's triangles
and the other train's, in whole numbers, and the tables the machines look things up in.

    python segments/train/data/make_data.py          # writes segments/train/data/scene.bin (not kept in git)

Whole numbers: places in centimetres (the S-curve's bend already in the world's, as the prototype bends it once);
normals in hundredths (99 long); albedos in thousandths. Each triangle: its three corners (indices into the corner
list), its three corner normals, its face normal and its plane's distance (the face normal's dot with a corner,
hundredths of a centimetre), its material, and what the prototype selects it by (the nearest and farthest z, the
middle's x, small, level of detail and the place of the model it belongs to, metres).

The file is a list of named arrays of 32-bit whole numbers: for each, the name's length and the name, the number of
dimensions and each one's size, then the numbers (little-endian).
"""
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "prototypes"))
sys.path.insert(0, os.path.join(HERE, "..", "..", "shared"))
import records                                            # noqa: E402
import train as T                                          # noqa: E402
import train_depart as TD                                  # noqa: E402

OUT = os.path.join(HERE, "scene.bin")
ARRAYS = []

def put(name, a):
    a = np.ascontiguousarray(np.asarray(a), dtype="<i4")
    ARRAYS.append((name, a))

def cm(v):
    return np.round(np.asarray(v, float) * 100).astype(np.int64)

def hundredths(n):
    n = np.asarray(n, float)
    ln = np.linalg.norm(n, axis=-1, keepdims=True)
    return np.round(n / np.maximum(ln, 1e-9) * 99).astype(np.int64)

def triangles(V, N, M, prefix):
    """Corners shared by place; per triangle its corners, normals, face normal and plane."""
    P = cm(V).reshape(-1, 3)
    uniq, inv = np.unique(P, axis=0, return_inverse=True)
    idx = inv.reshape(-1, 3)
    vn = hundredths(N)
    fnf = np.cross(V[:, 1] - V[:, 0], V[:, 2] - V[:, 0])
    flip = np.sum(fnf * N.sum(1), -1) < 0              # as the prototype: the face's normal agrees with its corners'
    fnf[flip] *= -1
    fn = hundredths(fnf)
    fd = np.sum(fn * cm(V[:, 0]), -1)                   # hundredths of a centimetre
    assert np.abs(fd).max() < 999_000_000 and np.abs(uniq).max() < 999_999
    put(prefix + "corners", uniq)
    put(prefix + "tris", idx)
    put(prefix + "vn", vn.reshape(len(V), 9))
    put(prefix + "fn", fn)
    put(prefix + "fd", fd)
    put(prefix + "mat", M)
    return idx

def wire_segments():
    """The wires as segments, as the prototype's wires() draws them (but fixed in the world, the first node at a
    5 m step rather than 1 m ahead of the cab): x0 y0 z0 x1 y1 z1 (cm), radius (mm), shine (thousandths) and kind:
    0 the contact wire and the messenger over both lines (every 5 m), 1 a dropper, 2 the farm road's power lines,
    3 the suburb's."""
    segs = []
    zs = np.arange(-100.0, T.END + 705.0, 5.0)
    intun = lambda z: (z > T.TUN0) & (z < T.TUN1)
    for xw in (0.0, T.X2):
        span = np.mod(zs, 50.0) / 50.0
        stagger = 0.22 * (1 - 2 * np.abs(np.mod(zs / 50.0, 2) - 1))
        cy = np.full_like(zs, 5.1)
        my = np.where(intun(zs), 5.9, 6.6 - 0.9 * 4 * span * (1 - span))
        P1 = np.stack([xw + stagger - 0.11 + T.bend(zs), cy, zs], 1)
        P2 = np.stack([xw + T.bend(zs), my, zs], 1)
        for P, r, sh in ((P1, 0.012, 0.02), (P2, 0.010, 0.015)):
            for k in range(len(zs) - 1):
                segs.append(list(cm(P[k])) + list(cm(P[k + 1])) + [round(r * 1000), round(sh * 1000), 0])
        for k in range(len(zs)):
            segs.append(list(cm(P1[k])) + list(cm(P2[k])) + [6, 20, 1])
    for zp0, zp1 in zip(T.POLES[:-1], T.POLES[1:]):
        zz = np.linspace(zp0, zp1, 9)
        u = (zz - zp0) / (zp1 - zp0)
        for dx in (-1.1, 0.0, 1.1):
            yy = T.LAND + 9.85 - 0.9 * 4 * u * (1 - u)
            P = np.stack([np.full_like(zz, T.MID - 19 + dx) + T.bend(zz), yy, zz], 1)
            for k in range(8):
                segs.append(list(cm(P[k])) + list(cm(P[k + 1])) + [12, 20, 2])
    for P, rad in T.CITY_WIRES:
        for k in range(len(P) - 1):
            segs.append(list(cm(P[k])) + list(cm(P[k + 1])) + [round(rad * 1000), 0, 3])
    # the departure's overhead line, behind the ride's start (after the ride's, which keep their places)
    zs = np.arange(TD.TOWN_FROM, -100.0 + 1e-6, 5.0)
    for xw in (0.0, T.X2):
        span = np.mod(zs, 50.0) / 50.0
        stagger = 0.22 * (1 - 2 * np.abs(np.mod(zs / 50.0, 2) - 1))
        P1 = np.stack([xw + stagger - 0.11, np.full_like(zs, 5.1), zs], 1)
        P2 = np.stack([np.full_like(zs, xw), 6.6 - 0.9 * 4 * span * (1 - span), zs], 1)
        for P, r, sh in ((P1, 0.012, 0.02), (P2, 0.010, 0.015)):
            for k in range(len(zs) - 1):
                segs.append(list(cm(P[k])) + list(cm(P[k + 1])) + [round(r * 1000), round(sh * 1000), 0])
        for k in range(len(zs) - 1):
            segs.append(list(cm(P1[k])) + list(cm(P2[k])) + [6, 20, 1])
    return np.array(segs)

def main():
    # the departure's station and town, after the ride's world (which keeps its triangles' places): how many are
    # the ride's
    n_ride = []
    def hook(g, mats):
        n_ride.append(len(g.M) + sum(len(c[2]) for c in g.chunks))
        TD.build_extra(g, mats)
    T.HOOKS["build"] = hook
    S = T.scene()
    put("w_ride", np.array(n_ride))
    V, N, M = S["V"], S["N"], S["M"]
    triangles(V, N, M, "w_")
    put("w_zlo", np.floor(S["zlo"]).astype(int))
    put("w_zhi", np.ceil(S["zhi"]).astype(int))
    put("w_xmid", np.round(S["xmid"]).astype(int))
    put("w_small", S["small"].astype(int))
    put("w_lod", S["lod"].astype(int))
    put("w_iz", np.where(S["iz"] < -1e8, -999999, np.round(S["iz"])).astype(int))
    triangles(S["TV"], S["TN"], S["TM"], "t_")
    kind, alb, par = S["mats"]
    put("m_kind", kind.astype(int))
    put("m_alb", np.round(alb * 1000).astype(int))
    put("m_par", np.round(par * 100).astype(int))          # the prototype's parameters, hundredths
    put("poles", np.round(np.array(T.POLES) * 100).astype(int))
    wires = [np.concatenate([cm(P).reshape(-1), [int(round(r * 1000))]]) for P, r in T.CITY_WIRES]
    put("city_wires", np.array(wires))
    put("wires", wire_segments())
    with open(OUT, "wb") as f:
        for name, a in ARRAYS:
            f.write(records.record(name, a))
    for name, a in ARRAYS:
        print(name, a.shape, a.min() if a.size else "", a.max() if a.size else "")

if __name__ == "__main__":
    main()
