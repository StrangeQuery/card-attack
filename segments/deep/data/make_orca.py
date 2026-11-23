#!/usr/bin/env python3
"""The orcas' models (the pair crossing the canyon's haze in the dive at 1536x768): the Killer Whale by Poly by
Google (CC BY 3.0, poly.pizza/m/7pqZEQ9b_E-; see CREDITS.md) as the mesh kit's cards, at 12 phases of its swim.

    python make_orca.py MODEL.glb       writes orca_0.txt .. orca_11.txt beside it

The model through the kit's own segments/mesh/data/make_mesh.py (Blender, headless): its space x across, y up, z
away, the head at -z, 860 units nose to flukes. Then bent for each phase k of the swim as the design prototype
(prototypes/orca.py, variant 6) bends it: a wave running back down the body, growing toward the flukes, sideways
(0.035 of its length at most, 1.1 waves a body length): x + 860 0.035 tail^1.6 sin(2 pi 1.1 u - 2 pi k / 12), u
how far toward the head (-z / 860), tail the part of the body behind 0.55 - u. Each phase's corners rounded to whole
units, its faces' normals and planes worked out again from them (as make_mesh.py does); the corners' normals kept
(a flat draw doesn't use them).
"""
import os
import subprocess
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
PHASES = 12
WAVE = 0.035
LENGTH = 860


def unit99(n):
    n = n / np.maximum(np.linalg.norm(n, axis=-1, keepdims=True), 1e-12)
    return np.rint(99 * n).astype(int)


def main():
    src = sys.argv[1]
    straight = os.path.join(HERE, "orca_straight.tmp.txt")
    subprocess.run([sys.executable, os.path.join(HERE, "..", "..", "mesh", "data", "make_mesh.py"), src, "0", straight], check=True)
    lines = open(straight).read().splitlines()
    head = [l for l in lines if l.startswith("#")]
    body = [l for l in lines if not l.startswith("#")]
    nv = int(body[1].split()[1])
    V = np.array([list(map(int, l.split()[:3])) for l in body[2:2 + nv]])
    N = [l.split()[3:6] for l in body[2:2 + nv]]
    tri_head = body[2 + nv]
    T = np.array([list(map(int, l.split()[:3])) for l in body[3 + nv:]])
    F0 = np.array([list(map(int, l.split()[3:6])) for l in body[3 + nv:]])
    assert V[:, 2].max() - V[:, 2].min() == LENGTH, V[:, 2].max() - V[:, 2].min()
    u = -V[:, 2] / LENGTH
    tail = np.clip(0.55 - u, 0, None)
    for k in range(PHASES):
        sway = LENGTH * WAVE * tail ** 1.6 * np.sin(2 * np.pi * 1.1 * u - 2 * np.pi * k / PHASES)
        B = V.copy()
        B[:, 0] = np.rint(V[:, 0] + sway).astype(int)
        a, b, c = B[T[:, 0]].astype(float), B[T[:, 1]].astype(float), B[T[:, 2]].astype(float)
        fn = np.cross(b - a, c - a)
        # a face bent to nothing keeps its straight normal; each the way the straight one points
        fn = np.where(np.linalg.norm(fn, axis=1, keepdims=True) < 1e-9, F0.astype(float), fn)
        fn = unit99(fn)
        flip = (fn * F0).sum(1) < 0
        fn[flip] = -fn[flip]
        fd = np.rint((fn * B[T[:, 0]]).sum(1) / 1000).astype(int)
        out = os.path.join(HERE, f"orca_{k}.txt")
        with open(out, "w") as f:
            f.write(head[0].replace("Made by make_mesh.py.", f"Made by make_mesh.py; swim phase {k} of {PHASES} by make_orca.py.") + "\n")
            f.write(body[0] + "\n" + body[1] + "\n")
            for p, q in zip(B, N):
                f.write(f"{p[0]} {p[1]} {p[2]} {q[0]} {q[1]} {q[2]}\n")
            f.write(" ".join(tri_head.split()[:2]) + " nouv\n")
            for t, n, d in zip(T, fn, fd):
                f.write(f"{t[0]} {t[1]} {t[2]} {n[0]} {n[1]} {n[2]} {d}\n")
        print(out)
    os.remove(straight)


if __name__ == "__main__":
    main()
