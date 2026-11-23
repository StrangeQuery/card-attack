"""Run inside Blender (headless): models the accelerated drive's meshes and writes them to prototypes/accel_models/.

    blender -b -P accel_models.py

* car: our car, a late-90s red wedge (the design's red, 176 26 30): a low body with a sloping hood, a cabin set
  back with dark glass, a rear wing on two posts, tail-lamp bars with grille slats between, a plate, a dark bumper,
  four wheels. Seen from behind as before, and from any angle in the jump.
* ramp: the obstacle, road works in our lane: a plywood ramp with yellow and black stripes up its slope, a striped
  barrier board each side on legs with an amber beacon on top, and three cones along its right.

Units are the drive's world units (the street 12 wide between kerbs); Blender's X is across, Y along (forward), Z
up. Each is written as .obj with its .mtl (a colour per material; the emissive ones say so in their names) and as
.glb for the mesh kit (segments/mesh, `blender_export.py` reads .glb).
"""
import math
import os

import bpy

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "accel_models")
os.makedirs(OUT, exist_ok=True)


def material(name, rgb):
    m = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    m.diffuse_color = (*(c / 255 for c in rgb), 1.0)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (*((c / 255) ** 2.2 for c in rgb), 1.0)
        if name.startswith("emit"):
            bsdf.inputs["Emission Color"].default_value = (*((c / 255) ** 2.2 for c in rgb), 1.0)
            bsdf.inputs["Emission Strength"].default_value = 3.0
    return m


def solid(name, verts, faces, mats, face_mats):
    """A mesh object from vertices, faces and a material index per face."""
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    for m in mats:
        me.materials.append(m)
    for p, k in zip(me.polygons, face_mats):
        p.material_index = k
    me.update()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def box(name, x0, x1, y0, y1, z0, z1, mat):
    v = [(x, y, z) for z in (z0, z1) for y in (y0, y1) for x in (x0, x1)]
    f = [(0, 2, 3, 1), (4, 5, 7, 6), (0, 1, 5, 4), (2, 6, 7, 3), (0, 4, 6, 2), (1, 3, 7, 5)]
    return solid(name, v, f, [mat], [0] * 6)


def prism(name, profile, x0, x1, mat):
    """A solid from a convex side profile (y, z), counter-clockwise seen from +X, between x0 and x1."""
    n = len(profile)
    v = [(x0, y, z) for y, z in profile] + [(x1, y, z) for y, z in profile]
    f = [tuple(reversed(range(n))), tuple(range(n, 2 * n))]
    f += [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    return solid(name, v, f, [mat], [0] * len(f))


def wheel(name, x, y, z, r, w, mat, sides=12):
    v = []
    for side in (x - w / 2, x + w / 2):
        for k in range(sides):
            a = 2 * math.pi * k / sides
            v.append((side, y + r * math.cos(a), z + r * math.sin(a)))
    f = [tuple(reversed(range(sides))), tuple(range(sides, 2 * sides))]
    f += [(k, (k + 1) % sides, sides + (k + 1) % sides, sides + k) for k in range(sides)]
    return solid(name, v, f, [mat], [0] * len(f))


def cone(name, x, y, mat, band, sides=8):
    v = [(x + 0.18 * math.cos(2 * math.pi * k / sides), y + 0.18 * math.sin(2 * math.pi * k / sides), 0.04) for k in range(sides)]
    v += [(x + 0.10 * math.cos(2 * math.pi * k / sides), y + 0.10 * math.sin(2 * math.pi * k / sides), 0.40) for k in range(sides)]
    v += [(x + 0.07 * math.cos(2 * math.pi * k / sides), y + 0.07 * math.sin(2 * math.pi * k / sides), 0.52) for k in range(sides)]
    v += [(x, y, 0.72)]
    f, m = [], []
    for k in range(sides):
        k1 = (k + 1) % sides
        f.append((k, k1, sides + k1, sides + k)); m.append(0)
        f.append((sides + k, sides + k1, 2 * sides + k1, 2 * sides + k)); m.append(1)
        f.append((2 * sides + k, 2 * sides + k1, 3 * sides)); m.append(0)
    base = box(name + "_base", x - 0.24, x + 0.24, y - 0.24, y + 0.24, 0.0, 0.04, mat)
    return [solid(name, v, f, [mat, band], m), base]


def join(objs, name):
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    objs[0].name = name
    return objs[0]


def export(ob, name):
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.wm.obj_export(filepath=os.path.join(OUT, name + ".obj"), export_selected_objects=True, export_materials=True,
                          export_triangulated_mesh=True, export_normals=False, export_uv=False, forward_axis="Y", up_axis="Z",
                          apply_modifiers=True)
    bpy.ops.export_scene.gltf(filepath=os.path.join(OUT, name + ".glb"), use_selection=True, export_format="GLB")


bpy.ops.wm.read_factory_settings(use_empty=True)

# ---- our car --------------------------------------------------------------------------------------------------
RED, DARK, GLASS = material("body", (176, 26, 30)), material("dark", (60, 14, 16)), material("glass", (30, 34, 50))
TAIL, BLACK, PLATE = material("emit_tail", (255, 60, 50)), material("black", (10, 10, 12)), material("plate", (220, 220, 200))
TYRE, HEAD, RIM = material("tyre", (14, 14, 16)), material("emit_head", (255, 250, 225)), material("rim", (120, 120, 128))
W = 0.96
parts = [prism("body", [(-2.10, 0.22), (2.05, 0.22), (2.16, 0.30), (2.12, 0.42), (0.62, 0.60), (-1.60, 0.66), (-2.05, 0.66),
                        (-2.10, 0.62)], -W, W, RED)]
# the cabin: a hull from the deck (wide) to the roof (narrow), glass all round but the roof
cv = [(x * s, y, z) for (s, y, z) in ((0.86, -1.55, 0.63), (0.86, 0.75, 0.61), (0.66, 0.05, 0.98), (0.66, -0.85, 0.98)) for x in (-1, 1)]
# indices: 0,1 rear-bottom (-x,+x), 2,3 front-bottom, 4,5 front-top, 6,7 rear-top
cf = [(0, 1, 7, 6), (3, 2, 4, 5), (6, 7, 5, 4), (0, 6, 4, 2), (1, 3, 5, 7), (0, 2, 3, 1)]
parts.append(solid("cabin", cv, cf, [GLASS, RED], [0, 0, 1, 0, 0, 0]))
parts.append(box("bumper", -0.97, 0.97, -2.15, -1.95, 0.18, 0.31, DARK))
parts.append(box("fbumper", -0.97, 0.97, 1.95, 2.18, 0.18, 0.28, DARK))
for x0, x1 in ((-0.93, -0.47), (0.47, 0.93)):
    parts.append(box("tail", x0, x1, -2.13, -2.05, 0.44, 0.55, TAIL))
    parts.append(box("hl", x0 + 0.05, x1 - 0.05, 2.08, 2.15, 0.31, 0.37, HEAD))
for k in range(4):
    z = 0.445 + 0.028 * k
    parts.append(box("slat", -0.36, 0.36, -2.12, -2.07, z, z + 0.012, BLACK))
parts.append(box("plate", -0.16, 0.16, -2.17, -2.12, 0.20, 0.30, PLATE))
# the rear wing on two posts
parts.append(box("wing", -0.92, 0.92, -2.08, -1.78, 0.80, 0.84, RED))
for x in (-0.6, 0.6):
    parts.append(box("post", x - 0.04, x + 0.04, -1.98, -1.90, 0.64, 0.80, DARK))
for x in (-0.84, 0.84):
    for y in (-1.38, 1.36):
        parts.append(wheel("tyre", x, y, 0.32, 0.32, 0.26, TYRE))
        parts.append(wheel("hub", x + (0.135 if x > 0 else -0.135), y, 0.32, 0.17, 0.01, RIM, 8))
car = join(parts, "car")
export(car, "car")
bpy.data.objects.remove(car)

# ---- the ramp and the road works --------------------------------------------------------------------------------
PLY, YEL, BLK = material("plywood", (150, 110, 62)), material("stripe_yellow", (235, 190, 30)), material("stripe_black", (24, 24, 26))
ORANGE, WHITE, AMBER = material("orange", (240, 110, 20)), material("white", (230, 230, 225)), material("emit_beacon", (255, 170, 40))
LEG = material("leg", (90, 90, 96))
RW_, L, H = 1.3, 3.0, 0.9
parts = [prism("ramp", [(0.0, 0.0), (L, 0.0), (L, H), (L - 0.02, H)], -RW_, RW_, PLY)]
# the stripes up the slope: thin panels just above it
n = 7
for k in range(n):
    y0, y1 = L * k / n, L * (k + 1) / n
    z0, z1 = H * y0 / L + 0.012, H * y1 / L + 0.012
    v = [(-RW_ + 0.02, y0, z0), (RW_ - 0.02, y0, z0), (RW_ - 0.02, y1, z1), (-RW_ + 0.02, y1, z1)]
    parts.append(solid("stripe", v, [(0, 1, 2, 3)], [YEL if k % 2 == 0 else BLK], [0]))
for side in (-1, 1):
    x = side * 1.9
    for k in range(4):     # the board, orange and white in four bands
        parts.append(box("board", x - 0.06, x + 0.06, 0.3 + 0.6 * k, 0.3 + 0.6 * (k + 1), 0.55, 0.85, ORANGE if k % 2 == 0 else WHITE))
    for y in (0.4, 2.6):
        parts.append(box("leg", x - 0.04, x + 0.04, y - 0.04, y + 0.04, 0.0, 0.55, LEG))
    parts.append(box("beacon", x - 0.09, x + 0.09, 1.41, 1.59, 0.85, 1.03, AMBER))
for k, (x, y) in enumerate(((1.7, -2.5), (1.7, -6.0), (1.7, -9.5))):
    parts += cone("cone", x, y, ORANGE, WHITE)
ramp = join(parts, "ramp")
export(ramp, "ramp")
print("written", OUT)
