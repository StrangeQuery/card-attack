"""Our car, the accelerated drive's mesh, as the mesh kit's cards (prototypes/accel_models/car.obj, made by
prototypes/accel_models.py in Blender).

    python make_car.py

Prints car.txt in make_mesh.py's form (the kit's model): the model's space x across (right), y up, z along (away),
100 units to the street's unit, the middle at its turn's pivot (0.5 up); each triangle's corners as the .obj winds
them (its normal out of the car), its face's normal in hundredths (99 long), its plane's distance, and an eighth
card: its material's number. And car_mats.txt: each material's name and its colour (8-bit: the .mtl's Kd to the
power 1 / 2.2, as the design takes it), ambient (30 hundredths of it), diffuse (78 hundredths: the design's 75 over
the kit's brightest light, 96 hundredths) and highlight (70); a lamp's glass (emit_*) its colour alone.
"""
import os

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
MODELS = os.path.join(HERE, "..", "..", "..", "prototypes", "accel_models")
PIVOT = 0.5


def main():
    mats, cur, order = {}, None, []
    for l in open(os.path.join(MODELS, "car.mtl")):
        p = l.split()
        if p and p[0] == "newmtl":
            cur = p[1]
            order.append(cur)
        elif p and p[0] == "Kd":
            mats[cur] = [round(float(v) ** (1 / 2.2) * 255) for v in p[1:4]]
    vs, tris, tm = [], [], []
    for l in open(os.path.join(MODELS, "car.obj")):
        p = l.split()
        if not p:
            continue
        if p[0] == "v":
            vs.append([float(v) for v in p[1:4]])
        elif p[0] == "usemtl":
            cur = p[1]
        elif p[0] == "f":
            idx = [int(q.split("/")[0]) - 1 for q in p[1:]]
            for k in range(1, len(idx) - 1):
                tris.append([idx[0], idx[k], idx[k + 1]])
                tm.append(order.index(cur))
    v = np.array(vs)
    # (across, along, up) to the model's (across, up, along), 100 to a unit, about the pivot
    m = np.stack([v[:, 0], v[:, 2] - PIVOT, v[:, 1]], 1) * 100
    vi = np.rint(m).astype(int)
    assert np.abs(vi).max() <= 499
    t = np.array(tris)
    a, b, c = m[t[:, 0]], m[t[:, 1]], m[t[:, 2]]
    # the swap (y and z) mirrors the model, so a winding's normal is the cross product taken the other way
    fn = -np.cross(b - a, c - a)
    fn = fn / np.maximum(np.linalg.norm(fn, axis=1, keepdims=True), 1e-12)
    out = (fn * (a + b + c - 3 * m.mean(0))).sum(1) > 0
    fni = np.rint(99 * fn).astype(int)
    fd = np.rint((fni * vi[t[:, 0]]).sum(1) / 1000).astype(int)
    # a corner's normal: its first triangle's (flat shading takes the faces')
    vn = np.zeros_like(vi)
    vn[:, 1] = 99
    for k in range(len(t) - 1, -1, -1):
        for i in t[k]:
            vn[i] = fni[k]
    with open(os.path.join(HERE, "car.txt"), "w") as f:
        f.write(f"# our car (prototypes/accel_models/car.obj): {len(t)} triangles, {len(vi)} corners; 100 units a "
                f"street's unit. Made by make_car.py.\n")
        f.write(f"faces {len(t)}\n")
        f.write(f"vertices {len(vi)}\n")
        for p, q in zip(vi, vn):
            f.write(f"{p[0]} {p[1]} {p[2]} {q[0]} {q[1]} {q[2]}\n")
        f.write(f"triangles {len(t)} nouv\n")
        for k in range(len(t)):
            f.write(f"{t[k][0]} {t[k][1]} {t[k][2]} {fni[k][0]} {fni[k][1]} {fni[k][2]} {fd[k]} {tm[k]}\n")
    with open(os.path.join(HERE, "car_mats.txt"), "w") as f:
        for name in order:
            col = mats[name]
            if name.startswith("emit_"):
                amb, dif, hi = col, [0, 0, 0], [0, 0, 0]
            else:
                amb, dif, hi = [round(0.30 * v) for v in col], [round(0.78 * v) for v in col], [70, 70, 70]
            f.write(f"{name} {' '.join(map(str, amb + dif + hi))}\n")
    print(f"car.txt: {len(t)} triangles, {len(vi)} corners; {out.mean():.0%} of the faces' normals point out of the "
          f"middle; car_mats.txt: {len(order)} materials")


if __name__ == "__main__":
    main()
