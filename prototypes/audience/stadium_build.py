"""The stadium party's solid geometry, built in Blender headless and baked to triangles.

    blender -b --factory-startup -P stadium_build.py -- OUTDIR

Writes OUTDIR/stadium.npz (corner positions T x 3 x 3, corner normals T x 3 x 3, a material per triangle) and
OUTDIR/stadium.json (the materials, the screen, the LED walls, the lights, the crowd's areas, the bowl). Metres,
Z up; the stage and the giant screen at the north end (+Y), the crowd facing it.

Only solid things are built here (the pitch, the bowl's tiers, its outer wall and canopy, the stage, the trusses,
the fixtures). The screen's picture, the LED walls, the crowd, the beams and lasers are placed by the prototype
(`prototypes/stadium.py`) from the JSON, so the screen's corners are exact.
"""
import json
import math
import os
import sys

import bpy
import bmesh
import numpy as np

args = sys.argv[sys.argv.index("--") + 1:]
OUT = args[0]
os.makedirs(OUT, exist_ok=True)


class Room:
    def __init__(self):
        bpy.ops.wm.read_factory_settings(use_empty=True)
        self.mats = []          # [{name, kind, rgb, ...}]
        self.objs = []          # (object, material index, smooth)
        self.info = {"lights": [], "decals": [], "areas": [], "seats": [], "emitters": []}

    def mat(self, name, rgb, kind="surf", **kw):
        for i, m in enumerate(self.mats):
            if m["name"] == name:
                return i
        self.mats.append(dict(name=name, rgb=list(rgb), kind=kind, **kw))
        return len(self.mats) - 1

    def _add(self, obj, mat, smooth=False, bevel=0.0, segs=2, subsurf=0):
        if bevel:
            m = obj.modifiers.new("b", "BEVEL")
            m.width = bevel
            m.segments = segs
            m.limit_method = "NONE"
        if subsurf:
            m = obj.modifiers.new("s", "SUBSURF")
            m.levels = subsurf
        self.objs.append((obj, mat, smooth))
        return obj

    def box(self, mat, x0, x1, y0, y1, z0, z1, bevel=0.0, smooth=False, segs=2, rot=0.0, piv=None):
        bpy.ops.mesh.primitive_cube_add(size=1)
        o = bpy.context.object
        o.scale = (x1 - x0, y1 - y0, z1 - z0)
        o.location = ((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
        if rot:
            px, py = piv if piv else (o.location.x, o.location.y)
            o.location = (px + (o.location.x - px) * math.cos(rot) - (o.location.y - py) * math.sin(rot),
                          py + (o.location.x - px) * math.sin(rot) + (o.location.y - py) * math.cos(rot), o.location.z)
            o.rotation_euler = (0, 0, rot)
        return self._add(o, mat, smooth or bevel > 0, bevel, segs)

    def cyl(self, mat, x, y, z0, z1, r, n=16, r2=None, smooth=True, rot=(0, 0, 0)):
        bpy.ops.mesh.primitive_cone_add(vertices=n, radius1=r, radius2=r if r2 is None else r2, depth=z1 - z0,
                                        location=(x, y, (z0 + z1) / 2), rotation=rot)
        return self._add(bpy.context.object, mat, smooth)

    def sphere(self, mat, x, y, z, r, seg=16, ring=8, sx=1, sy=1, sz=1):
        bpy.ops.mesh.primitive_uv_sphere_add(segments=seg, ring_count=ring, radius=r, location=(x, y, z))
        o = bpy.context.object
        o.scale = (sx, sy, sz)
        return self._add(o, mat, True)

    def rod(self, mat, a, b, r, n=8):
        a, b = np.array(a, float), np.array(b, float)
        d = b - a
        L = float(np.linalg.norm(d))
        bpy.ops.mesh.primitive_cylinder_add(vertices=n, radius=r, depth=L, location=tuple((a + b) / 2))
        o = bpy.context.object
        o.rotation_mode = "QUATERNION"
        from mathutils import Vector
        o.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(Vector(tuple(d / L)))
        return self._add(o, mat, True)

    # --- the screen ---------------------------------------------------------------------------------------------
    def screen(self, cx, cy, cz, w, aspect):
        """The picture's rectangle, facing -Y (toward the crowd), centred at (cx, cy, cz); a black frame round it."""
        h = w / aspect
        self.info["screen"] = dict(centre=[cx, cy - 0.05, cz], right=[1, 0, 0], up=[0, 0, 1], width=w, height=h)
        m = self.mat("frame_black", (0.03, 0.03, 0.035))
        b = w * 0.015
        self.box(m, cx - w / 2 - b, cx + w / 2 + b, cy, cy + 1.0, cz - h / 2 - b, cz + h / 2 + b)
        return h

    def light(self, **kw):
        self.info["lights"].append(kw)

    def decal(self, image, cx, cy, cz, w, h, normal="-y", emit=0.0):
        self.info["decals"].append(dict(image=image, centre=[cx, cy, cz], width=w, height=h, normal=normal,
                                        emit=emit))

    def export(self):
        dg = bpy.context.evaluated_depsgraph_get()
        P, N, M = [], [], []
        for o, mi, smooth in self.objs:
            ev = o.evaluated_get(dg)
            me = ev.to_mesh()
            me.calc_loop_triangles()
            mw = o.matrix_world
            nm = mw.to_3x3().inverted().transposed()
            vs = np.array([tuple(mw @ v.co) for v in me.vertices])
            tri = np.array([tuple(t.vertices) for t in me.loop_triangles])
            p = vs[tri]
            if smooth:
                cn = np.array([tuple((nm @ v.normal).normalized()) for v in me.vertices])
                n = cn[tri]
            else:
                fn = np.array([tuple((nm @ t.normal).normalized()) for t in me.loop_triangles])
                n = np.repeat(fn[:, None, :], 3, axis=1)
            P.append(p)
            N.append(n)
            M.append(np.full(len(tri), mi))
            ev.to_mesh_clear()
        P, N, M = np.concatenate(P), np.concatenate(N), np.concatenate(M)
        np.savez_compressed(f"{OUT}/stadium.npz", p=P.astype(np.float32), n=N.astype(np.float32), m=M)
        self.info["materials"] = self.mats
        json.dump(self.info, open(f"{OUT}/stadium.json", "w"), indent=1)
        print("STADIUM", len(M), "triangles")


def truss(r, x0, x1, y, z, s=0.4):
    m = r.mat("truss", (0.55, 0.55, 0.58))
    for dy in (-s / 2, s / 2):
        for dz in (-s / 2, s / 2):
            r.rod(m, (x0, y + dy, z + dz), (x1, y + dy, z + dz), 0.03, n=6)
    n = max(2, int((x1 - x0) / s))
    for k in range(n):
        xa = x0 + (x1 - x0) * k / n
        xb = x0 + (x1 - x0) * (k + 1) / n
        r.rod(m, (xa, y - s / 2, z - s / 2), (xb, y - s / 2, z + s / 2), 0.012, n=4)
        r.rod(m, (xa, y + s / 2, z - s / 2), (xb, y + s / 2, z + s / 2), 0.012, n=4)


def moving_head(r, x, y, z):
    m = r.mat("fixture", (0.06, 0.06, 0.07))
    r.box(m, x - 0.15, x + 0.15, y - 0.15, y + 0.15, z - 0.12, z)
    r.sphere(m, x, y, z - 0.3, 0.17, seg=10, ring=6)


def stage(r, x0, x1, y0, y1, h):
    r.box(r.mat("stage", (0.08, 0.08, 0.09)), x0, x1, y0, y1, 0, h)


def stadium(r):
    """The bowl (an oval of stepped tiers, open at the north end for the stage), its outer wall and canopy, the
    pitch, the stage with its towers and truss, the giant screen; the lights and the crowd's areas as data."""
    A0, B0 = 78.0, 58.0          # the tiers' inner edge (half sizes across and along)
    ROWS, RUN, RISE = 30, 1.8, 1.0
    OPEN_Y = 38.0                # tiers stop short of the stage end
    r.box(r.mat("ground", (0.05, 0.05, 0.05)), -1500, 1500, -1500, 1500, -0.3, -0.2)
    r.box(r.mat("pitch", (0.02, 0.022, 0.02)), -A0, A0, -B0, OPEN_Y + 30, -0.2, 0)
    tm = r.mat("tier", (0.20, 0.18, 0.22))
    wall = r.mat("facade", (0.30, 0.30, 0.34))
    roof = r.mat("canopy", (0.22, 0.22, 0.25))
    strip = r.mat("canopy_lights", (0.75, 0.85, 1.0), "emit", emit=3.0)
    band = r.mat("facade_lights", (1.0, 0.85, 0.6), "emit", emit=1.5)
    n = 72

    def ring(k):
        return A0 + k * RUN, B0 + k * RUN

    def seg(k, j):
        rx, ry = ring(k)
        am = 2 * math.pi * (j + 0.5) / n
        xa, ya = rx * math.cos(am), ry * math.sin(am)
        L = 2 * math.pi * math.hypot(rx * math.sin(am), ry * math.cos(am)) / n
        rot = math.atan2(ry * math.cos(am), -rx * math.sin(am))
        return xa, ya, L, rot

    for k in range(ROWS + 2):
        for j in range(n):
            xa, ya, L, rot = seg(k, j)
            if ya > OPEN_Y:
                continue
            if k < ROWS:                    # a step: a box from the ground to the row's height
                z = 1.0 + k * RISE
                r.box(tm, xa - L / 2 - 0.15, xa + L / 2 + 0.15, ya - RUN / 2, ya + RUN / 2, 0, z, rot=rot, piv=(xa, ya))
            elif k == ROWS + 1:             # the outer wall
                r.box(wall, xa - L / 2 - 0.2, xa + L / 2 + 0.2, ya - 0.6, ya + 0.6, 0, 36.0, rot=rot, piv=(xa, ya))
                for zb in (9.0, 20.0, 31.0):  # the concourses' lit windows, bands round the outside
                    r.box(band, xa - L / 2, xa + L / 2, ya - 0.7, ya + 0.7, zb, zb + 1.2, rot=rot, piv=(xa, ya))
    for j in range(n):                      # the canopy over the upper rows, its lit inner edge
        xa, ya, L, rot = seg(ROWS - 9, j)
        xb, yb, Lb, _ = seg(ROWS + 1, j)
        if ya > OPEN_Y or yb > OPEN_Y:
            continue
        if ya < -10 and abs(xa) < 80:       # cut back over the south end, where the night arrival crests the rim:
            continue                         # the bowl opens unobstructed the moment the camera crosses
        cx, cy = (xa + xb) / 2, (ya + yb) / 2
        depth = math.hypot(xb - xa, yb - ya)
        Lm = (L + Lb) / 2
        r.box(roof, cx - Lm / 2 - 0.3, cx + Lm / 2 + 0.3, cy - depth / 2, cy + depth / 2, 37.0, 38.2, rot=rot,
              piv=(cx, cy))
        r.box(strip, xa - L / 2, xa + L / 2, ya - 0.2, ya + 0.2, 36.7, 37.0, rot=rot, piv=(xa, ya))
    r.info["bowl"] = dict(A0=A0, B0=B0, rows=ROWS, run=RUN, rise=RISE, z0=1.0, open_y=OPEN_Y, segments=n,
                          canopy_from=ROWS - 9, canopy_z=37.0)

    # the stage end: the stage, a wall behind, the giant screen, two LED walls, the towers and the truss
    stage(r, -70, 70, 40.0, 66.0, 3.0)
    r.box(r.mat("stage_wall", (0.05, 0.05, 0.06)), -95, 95, 64.0, 66.0, 0, 64.0)
    r.screen(0.0, 63.9, 33.0, 96.0, 2.0)
    r.info["led_walls"] = [dict(centre=[-72, 63.8, 26.0], width=24, height=44),
                           dict(centre=[72, 63.8, 26.0], width=24, height=44)]
    tw = r.mat("tower", (0.25, 0.25, 0.27))
    for x in (-58.0, 58.0):
        for y in (44.0, 60.0):
            r.box(tw, x - 1.2, x + 1.2, y - 1.2, y + 1.2, 3.0, 62.0)
    truss(r, -60, 60, 44.0, 60.0, 2.0)
    truss(r, -60, 60, 52.0, 61.5, 2.0)
    for k, x in enumerate(np.linspace(-52, 52, 14)):
        moving_head(r, float(x), 44.0, 58.6)
        r.light(type="spot", pos=[float(x), 44.0, 58.0], cone=5,
                rgb=[[1, 0.85, 0.7], [0.3, 0.5, 1], [1, 0.25, 0.6], [0.3, 1, 0.5]][k % 4], power=1.0, phase=k * 0.45)
    for k, x in enumerate(np.linspace(-66, 66, 8)):          # searchlights on the stage's front edge
        r.light(type="search", pos=[float(x), 41.0, 3.5], rgb=[1, 1, 0.95], power=1.0, phase=k * 0.9)
    for k in range(10):                                       # searchlights on the canopy, round the bowl
        am = -math.pi / 2 + (k - 4.5) * 0.42
        rx, ry = ring(ROWS - 4)
        r.light(type="search", pos=[rx * math.cos(am), ry * math.sin(am), 38.5], rgb=[0.85, 0.9, 1.0], power=0.8,
                phase=k * 1.1 + 0.3)
    for x, c in ((0, [0.2, 1.0, 0.3]), (-40, [1.0, 0.1, 0.4]), (40, [0.3, 0.4, 1.0]), (-20, [1, 0.8, 0.1]),
                 (20, [0.1, 0.9, 1.0])):
        r.light(type="lasers", pos=[float(x), 41.5, 4.0], n=12, rgb=c, spread=100)
    r.info["pyro"] = [[float(x), 41.0, 3.0] for x in np.linspace(-60, 60, 8)]
    r.info["areas"] = [dict(kind="pitch", x0=-A0 + 2, x1=A0 - 2, y0=-B0 + 2, y1=36.0)]


r = Room()
stadium(r)
r.export()
