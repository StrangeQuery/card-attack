"""The mesh kit's data: a model as cards. Prints the triangle list the host deals and the reference reads.

    python make_mesh.py SOURCE LEVEL OUT.txt [--radius R]
    python make_mesh.py --spec POWER spec.txt      # the highlight's table


SOURCE is `suzanne` (Blender's monkey), a .glb/.gltf/.obj file, or a .blend (its meshes); LEVEL the Catmull-Clark
subdivision level (0 for the model as it is). Blender (headless, `blender -b`) exports the triangles through
`blender_export.py`; this script turns them into whole numbers:

- the model's space: x right, y up, z away from the viewer (Blender's x, z and y: its front, -y, faces the camera);
  centred on its box and scaled so no corner is further than R (499 by default) from the middle, rounded to whole
  units: every coordinate a card, and any turn of the model too;
- each corner's normal (Blender's, smooth) in hundredths of a unit, 99 long;
- each triangle: its corners, its face's normal (its polygon's, so a quad's two halves share it) in hundredths, and
  its plane's distance from the middle along that normal (the sum of the normal's parts times a corner's over 1000,
  rounded): what the back-face test compares the camera with; then its corners' UVs in thousandths (0..999, v down)
  when the model has them.

Geometry is prepared here once, as an asset (like a font or a photo); every number worked out from it each frame
(turns, the camera, the light, the pixels) is CARDIAC's.
"""
import os
import subprocess
import sys
import tempfile

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
BLENDER = os.environ.get("BLENDER", "/opt/homebrew/bin/blender")


def export(source, level):
    tmp = tempfile.mkdtemp(prefix="mesh_")
    prefix = os.path.join(tmp, "m")
    cmd = [BLENDER, "-b"]
    src = source
    if source.endswith(".blend"):
        cmd.append(source)
        src = "scene"
    cmd += ["-P", os.path.join(HERE, "blender_export.py"), "--", prefix, src, str(level)]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL)
    return np.load(f"{prefix}_{level}.npz")


def unit99(n):
    n = n / np.maximum(np.linalg.norm(n, axis=-1, keepdims=True), 1e-12)
    return np.rint(99 * n).astype(int)


def spec_table(power, out):
    """The highlight for a dot of the half-way vector (0..999; a unit normal's at most 969: 99 hundredths times 979,
    the kit's half-way vector at 98 hundredths of 999): 99 (h / 969)^power, rounded, at most 99."""
    vals = [int(round(99 * min(1, h / 969) ** power)) for h in range(1000)]
    with open(out, "w") as f:
        f.write(f"# the highlight, 99 (h / 969)^{power} for h = 0..999 (the half-way vector's dot), at most 99. make_mesh.py --spec {power}\n")
        for i in range(0, 1000, 20):
            f.write(" ".join(str(v) for v in vals[i:i + 20]) + "\n")


def main():
    args = sys.argv[1:]
    if args[0] == "--spec":
        spec_table(int(args[1]), args[2])
        return
    radius = 499
    if "--radius" in args:
        i = args.index("--radius")
        radius = int(args[i + 1])
        del args[i:i + 2]
    source, level, out = args[0], int(args[1]), args[2]
    d = export(source, level)
    # Blender's (x, y, z) to the model's (x, z, y): y up, its front toward the camera
    swap = lambda a: a[..., [0, 2, 1]]
    v = swap(d["vertices"])
    n = swap(d["normals"])
    t = d["triangles"]
    centre = (v.max(0) + v.min(0)) / 2
    scale = radius / np.linalg.norm(v - centre, axis=1).max()
    vi = np.rint((v - centre) * scale).astype(int)
    assert np.all(np.linalg.norm(vi, axis=1) <= radius + 1)
    # the faces' normals: each polygon's triangles' cross products summed (the swap turns the winding round, so
    # the cross product is taken the other way)
    a, b, c = v[t[:, 0]], v[t[:, 1]], v[t[:, 2]]
    fn = -np.cross(b - a, c - a)
    pn = np.zeros((int(d["polygons"]), 3))
    np.add.at(pn, d["polygon"], fn)
    fn = unit99(pn[d["polygon"]])
    # the faces' normals point the way the corners' do (out of the model)
    flip = (fn * n[t].sum(1)).sum(1) < 0
    assert flip.mean() < 0.01, flip.mean()
    fd = np.rint((fn * vi[t[:, 0]]).sum(1) / 1000).astype(int)
    vn = unit99(n)
    uv = d["uvs"] if "uvs" in d else None
    with open(out, "w") as f:
        f.write(f"# {os.path.basename(source)} at subdivision level {level}: {int(d['polygons'])} faces, "
                f"{len(t)} triangles, {len(vi)} corners; radius {radius}. Made by make_mesh.py.\n")
        f.write(f"faces {int(d['polygons'])}\n")
        f.write(f"vertices {len(vi)}\n")
        for p, q in zip(vi, vn):
            f.write(f"{p[0]} {p[1]} {p[2]} {q[0]} {q[1]} {q[2]}\n")
        f.write(f"triangles {len(t)} {'uv' if uv is not None else 'nouv'}\n")
        for k in range(len(t)):
            # the swap mirrors the model, so the corners go the other way round to keep the winding
            i0, i1, i2 = t[k][0], t[k][2], t[k][1]
            line = f"{i0} {i1} {i2} {fn[k][0]} {fn[k][1]} {fn[k][2]} {fd[k]}"
            if uv is not None:
                u = uv[k][[0, 2, 1]]
                uu = np.clip(np.rint(u[:, 0] * 999), 0, 999).astype(int)
                vv = np.clip(np.rint((1 - u[:, 1]) * 999), 0, 999).astype(int)
                line += " " + " ".join(f"{uu[j]} {vv[j]}" for j in range(3))
            f.write(line + "\n")
    print(f"{out}: {len(vi)} corners, {len(t)} triangles, {int(d['polygons'])} faces, scale {scale:.2f} per unit")


if __name__ == "__main__":
    main()
