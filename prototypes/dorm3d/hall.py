"""The screening hall in 3D ('the same treatment' as the dorm): the hall between the drive and the
dive, its crowd (MakeHuman via MPFB 2, CC0 assets and poses: sixteen people posed, frozen and stood round the hall as
130), the projector on its cart with its beam through the haze, rendered ONCE in Cycles through screening.py's camera.
screening.py lays the renders out as its layers; nothing here is per frame.

    blender -b -P prototypes/dorm3d/hall.py -- OUTDIR TEXDIR [SCALE] [SAMPLES] [BLEND]

Writes OUTDIR/hall_{mon,beam,const}.exr (the whole frame at SCALE x 1536 x 768): 'mon' lit by the screen's picture
alone (white, an area light), 'beam' by the projector alone (its beam in the haze, what it catches), 'const' by
everything else that glows (the EXIT signs, the party's monitors along the walls, the projector's lens); with the
diffuse colour and the object index (the picture is index 1). The eye 1.6 m up at the back among them, f 1560, the
horizon on row 345; the front wall 15 m off, the hall 12 m wide and 4.5 m high, the screen 5 x 2.5 m from 1.3 m up.
"""
import math
import os
import random
import re
import sys

import bpy
from mathutils import Matrix, Vector

argv = sys.argv[sys.argv.index("--") + 1:]
OUTDIR, TEX = argv[0], argv[1]
SCALE = int(argv[2]) if len(argv) > 2 else 2
SAMPLES = int(argv[3]) if len(argv) > 3 else 256
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kit import *  # noqa: E402,F403
import kit  # noqa: E402
F, HZ, EY = 1560.0, 345.0, 2.4      # on a riser at the back: we look over every head at the screen
FD, L_, R_, CEIL = 15.0, -6.0, 6.0, 5.3
LENS = (-0.2, 1.45, 6.5)
SX0, SX1, SY0, SY1 = -2.5, 2.5, 2.0, 4.5      # the screen
sc = kit.init(F, HZ, EY)
os.makedirs(OUTDIR, exist_ok=True)


def mat_glow_image(name, path, strength):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree; b = nt.nodes["Principled BSDF"]
    b.inputs["Specular IOR Level"].default_value = 0.0
    tx = nt.nodes.new("ShaderNodeTexImage"); tx.image = bpy.data.images.load(path)
    nt.links.new(tx.outputs["Color"], b.inputs["Base Color"]); nt.links.new(tx.outputs["Color"], b.inputs["Emission Color"])
    b.inputs["Emission Strength"].default_value = strength
    return m


# ---- the hall ------------------------------------------------------------------------------------------------------
carpet = mat_noise("carpet", (0.05, 0.045, 0.05), (0.09, 0.08, 0.085), scale=300, bump=0.3)
panel = mat_brick("panels", (0.16, 0.14, 0.18), (0.05, 0.05, 0.06), 1.2, 1.2, bump=0.4)          # fabric wall panels
black = mat_noise("black", (0.02, 0.02, 0.025), (0.035, 0.035, 0.04), scale=80)
drape = mat_noise("drape", (0.03, 0.025, 0.035), (0.05, 0.045, 0.06), scale=200, bump=0.2)
steel = mat("steel", (0.18, 0.18, 0.2))
box("floor", L_, R_, -0.05, 0, -1.0, FD, carpet)
box("left", L_ - 0.1, L_, 0, CEIL, -1.0, FD, panel)
box("right", R_, R_ + 0.1, 0, CEIL, -1.0, FD, panel)
box("ceiling", L_, R_, CEIL, CEIL + 0.1, -1.0, FD, black)
box("front", L_, R_, 0, CEIL, FD, FD + 0.1, black)
box("back", L_, R_, 0, CEIL, -1.1, -1.0, panel)
# the drapes across the front wall, folded (a sine across them), round the screen
def drape_wall(name, x0, x1, y0, y1, d, folds=0.11):
    n = max(8, int((x1 - x0) / 0.04)); vs, fs = [], []
    for i in range(n + 1):
        x = x0 + (x1 - x0) * i / n
        dd = d - 0.06 - 0.05 * math.sin(2 * math.pi * x / folds * 0.5)
        vs += [V(x, y0, dd), V(x, y1, dd)]
        if i:
            fs.append((2 * i - 2, 2 * i, 2 * i + 1, 2 * i - 1))
    ob = mesh_ob(name, vs, fs, drape)
    for p in ob.data.polygons:
        p.use_smooth = True
    return ob
drape_wall("drape_l", L_, SX0 - 0.15, 0, CEIL, FD)
drape_wall("drape_r", SX1 + 0.15, R_, 0, CEIL, FD)
drape_wall("drape_top", SX0 - 0.15, SX1 + 0.15, SY1 + 0.15, CEIL, FD)
drape_wall("drape_low", SX0 - 0.15, SX1 + 0.15, 0, SY0 - 0.15, FD)
# the screen: its black border, the picture (index 1)
box("screen_frame", SX0 - 0.15, SX1 + 0.15, SY0 - 0.15, SY1 + 0.15, FD - 0.08, FD - 0.02, black)
screen = quad("picture", [(SX0, SY1, FD - 0.085), (SX1, SY1, FD - 0.085), (SX1, SY0, FD - 0.085), (SX0, SY0, FD - 0.085)], mat("screen", (0.6, 0.6, 0.6)))
screen.pass_index = 1
# the stage: a low riser under the screen, speakers stacked each side (cabinets, the woofers' cones and the horns)
box("stage", -5.0, 5.0, 0, 0.6, FD - 2.2, FD - 0.1, mat_noise("stage", (0.04, 0.035, 0.035), (0.06, 0.05, 0.05), 120), bevel=0.01)
box("stage_edge", -5.0, 5.0, 0.56, 0.6, FD - 2.22, FD - 2.2, steel)
cab = mat_noise("cab", (0.03, 0.03, 0.03), (0.05, 0.05, 0.05), scale=400, bump=0.3)
cone = mat("cone", (0.015, 0.015, 0.015))
for x0 in (-4.3, 3.3):
    for j, (h0, h1) in enumerate(((0.6, 1.5), (1.5, 2.4), (2.4, 3.0))):
        box("speaker", x0, x0 + 1.0, h0, h1 - 0.02, FD - 1.2, FD - 0.5, cab, bevel=0.01)
        if j < 2:
            for k_ in (0, 1):
                c = cyl("woofer", 0, 0, 0.03, 0, 0.18, cone, n=32, r1=0.12)
                c.matrix_world = Matrix.Translation(V(x0 + 0.27 + 0.46 * k_, (h0 + h1) / 2, FD - 1.2)) @ Matrix.Rotation(math.radians(90), 4, "X")
        else:
            box("horn", x0 + 0.2, x0 + 0.8, h0 + 0.15, h1 - 0.15, FD - 1.21, FD - 1.19, cone)
# the ceiling: trusses across, a stage light on each (off)
for d in (4.0, 7.0, 10.0, 13.0):
    for (y0, y1) in ((CEIL - 0.35, CEIL - 0.32), (CEIL - 0.05, CEIL - 0.02)):
        box("truss", L_, R_, y0, y1, d - 0.02, d + 0.02, steel)
    for i in range(24):
        x = L_ + i * 0.5
        box("truss_web", x, x + 0.02, CEIL - 0.35, CEIL - 0.02, d - 0.02, d + 0.02, steel)
    for x in (-3.0, -1.0, 1.0, 3.0):
        c = cyl("can_light", x, CEIL - 0.75, CEIL - 0.35, d, 0.1, black)
# the side doors with their EXIT signs; a banner along the left wall
exit_m = mat_glow_image("exit", os.path.join(TEX, "exit.png"), 3.0)
for X in (L_, R_):
    s_ = 1 if X < 0 else -1
    box("door", X - 0.03, X + 0.03, 0, 2.2, 11.0, 12.2, mat_wood("hdoor", (0.12, 0.08, 0.05), (0.16, 0.1, 0.06), 3))
    xs = X + 0.05 * s_
    pts = [(xs, 2.62, 11.25), (xs, 2.62, 11.95), (xs, 2.36, 11.95), (xs, 2.36, 11.25)]
    q_ = quad("exit_sign", pts if X > 0 else [pts[1], pts[0], pts[3], pts[2]], exit_m)
box("banner", L_ + 0.01, L_ + 0.02, 3.4, 4.0, 4.0, 12.0, mat_noise("banner", (0.15, 0.08, 0.35), (0.2, 0.12, 0.45), 30))
box("banner_r", R_ - 0.02, R_ - 0.01, 3.4, 4.0, 4.0, 12.0, mat_noise("banner2", (0.35, 0.08, 0.1), (0.45, 0.12, 0.14), 30))
# the party along the walls: tables, the monitors on them glowing (it's a demo party), chairs pushed in
table = mat_noise("table", (0.25, 0.25, 0.27), (0.3, 0.3, 0.32), 200)
beige = mat_noise("beige", (0.5, 0.47, 0.4), (0.55, 0.52, 0.44), scale=500)
GLOWS = []
rp = random.Random(4)
for X in (L_ + 0.45, R_ - 0.45):
    s_ = -1 if X < 0 else 1
    box("table", X - 0.4, X + 0.4, 0.72, 0.75, 2.5, 10.5, table)
    for d in (2.6, 10.4):
        for dx in (-0.35, 0.33):
            box("tleg", X + dx, X + dx + 0.03, 0, 0.72, d - 0.02, d + 0.01, steel)
    for i, d in enumerate((3.0, 4.3, 5.6, 6.9, 8.2, 9.5)):
        mx = X + 0.05 * s_
        box("party_crt", mx - 0.2, mx + 0.2, 0.75, 1.1, d - 0.2, d + 0.2, beige, bevel=0.01)
        col = rp.choice(((0.2, 0.4, 1.0), (0.1, 0.9, 0.5), (1.0, 0.4, 0.2), (0.7, 0.3, 1.0)))
        g = mat(f"party_glow{i}{s_}", (0.05, 0.05, 0.05), emit=col, strength=2.0)
        GLOWS.append(g)
        fx = mx - 0.201 * s_
        quad("party_pic", [(fx, 1.05, d + 0.15 * s_), (fx, 1.05, d - 0.15 * s_), (fx, 0.82, d - 0.15 * s_), (fx, 0.82, d + 0.15 * s_)], g)
        box("chair_seat", X - 0.75 * s_ - 0.2, X - 0.75 * s_ + 0.2, 0.44, 0.48, d - 0.2, d + 0.2, black)
        box("chair_back", X - 0.95 * s_ - 0.02, X - 0.95 * s_ + 0.02, 0.48, 0.9, d - 0.2, d + 0.2, black)

# the projector on its cart (a little left, 5.6 m off... its lens at LENS): the cart, a shelf, legs and casters, the
# projector, its lens (glowing a little), a laptop on the shelf
cx, cz = LENS[0], LENS[2] - 0.35
CT = 1.22
box("cart_top", cx - 0.35, cx + 0.35, CT - 0.04, CT, cz - 0.3, cz + 0.3, steel, bevel=0.005)
box("cart_shelf", cx - 0.35, cx + 0.35, 0.4, 0.43, cz - 0.3, cz + 0.3, steel)
for dx in (-0.33, 0.3):
    for dz in (-0.28, 0.25):
        box("cart_leg", cx + dx, cx + dx + 0.03, 0.06, CT - 0.04, cz + dz, cz + dz + 0.03, steel)
        cyl("caster", cx + dx + 0.015, 0, 0.06, cz + dz + 0.015, 0.03, black)
pj = mat_noise("projector", (0.12, 0.12, 0.13), (0.16, 0.16, 0.17), 300)
box("projector", cx - 0.2, cx + 0.2, CT, CT + 0.17, cz - 0.15, cz + 0.27, pj, bevel=0.015)
for i in range(6):
    box("pj_vent", cx - 0.21, cx - 0.2, CT + 0.03, CT + 0.14, cz - 0.1 + i * 0.05, cz - 0.08 + i * 0.05, black)
LENS_M = mat("lens", (0.1, 0.1, 0.1), emit=(1, 1, 0.95), strength=4.0)
GLOWS.append(LENS_M)
lens_barrel = cyl("lens_barrel", 0, 0, 0.08, 0, 0.045, black, n=24)
lens_barrel.matrix_world = Matrix.Translation(V(LENS[0], LENS[1], LENS[2] - 0.08)) @ Matrix.Rotation(math.radians(90), 4, "X")
lens_glass = cyl("lens_glass", 0, 0, 0.005, 0, 0.035, LENS_M, n=24)
lens_glass.matrix_world = Matrix.Translation(V(LENS[0], LENS[1], LENS[2])) @ Matrix.Rotation(math.radians(90), 4, "X")
box("laptop", cx - 0.17, cx + 0.17, 0.43, 0.45, cz - 0.12, cz + 0.12, mat("laptop", (0.1, 0.1, 0.11)))
cables = tube("cable", [(cx + 0.1, CT + 0.05, cz + 0.27), (cx + 0.15, 0.9, cz + 0.35), (cx + 0.1, 0.45, cz + 0.2)], 0.006, black)

# ---- the crowd: sixteen people, posed and frozen, stood round the hall as 130 (loose, standing, facing the screen;
# clear round the cart and of the line from us to it, as screening.py's flat crowd) ---------------------------------
CAST = [
    dict(gender=1.0, weight=0.5, race=(0.1, 0.8, 0.1), skin="young_caucasian_male", hair="short02", clothes=["male_casualsuit01"], pose="callharvey3d_standingdefault"),
    dict(gender=0.0, weight=0.45, race=(0.1, 0.8, 0.1), skin="young_caucasian_female", hair="long01", clothes=["female_casualsuit01"], pose="callharvey3d_standingnatural"),
    dict(gender=1.0, weight=0.65, race=(0.0, 0.2, 0.8), skin="young_african_male", hair="short01", clothes=["male_casualsuit02"], pose="drednicolson_arms_akimbo"),
    dict(gender=1.0, weight=0.45, race=(0.8, 0.1, 0.1), skin="young_asian_male", hair="short03", clothes=["male_casualsuit03"], pose="standing02"),
    dict(gender=0.0, weight=0.5, race=(0.8, 0.1, 0.1), skin="young_asian_female", hair="bob01", clothes=["female_casualsuit02"], pose="sohh_standing1"),
    dict(gender=1.0, weight=0.55, race=(0.1, 0.8, 0.1), skin="young_caucasian_male2", hair="short04", clothes=["male_casualsuit04"], pose="mindfront_standing_holding_wine_glass"),
    dict(gender=0.0, weight=0.55, race=(0.0, 0.1, 0.9), skin="young_african_female", hair="afro01", clothes=["female_sportsuit01"], pose="callharvey3d_standingatease"),
    dict(gender=1.0, weight=0.5, race=(0.1, 0.8, 0.1), skin="young_caucasian_male", hair="ponytail01", clothes=["male_casualsuit05"], pose="standing04"),
    dict(gender=1.0, weight=0.7, race=(0.3, 0.6, 0.1), skin="middleage_caucasian_male", hair="short02", clothes=["male_worksuit01"], pose="anrico_standing10"),
    dict(gender=0.0, weight=0.4, race=(0.1, 0.8, 0.1), skin="young_caucasian_female2", hair="ponytail01", clothes=["female_casualsuit01"], pose="standing03"),
    dict(gender=1.0, weight=0.5, race=(0.0, 0.3, 0.7), skin="young_african_male", hair="short04", clothes=["male_casualsuit06"], pose="standing06"),
    dict(gender=1.0, weight=0.45, race=(0.1, 0.8, 0.1), skin="young_caucasian_male2", hair="long01", clothes=["male_casualsuit02"], pose="standing01"),
    dict(gender=0.0, weight=0.5, race=(0.6, 0.3, 0.1), skin="young_asian_female", hair="braid01", clothes=["female_sportsuit01"], pose="callharvey3d_standingmodest"),
    dict(gender=1.0, weight=0.6, race=(0.6, 0.3, 0.1), skin="young_asian_male", hair="short01", clothes=["male_casualsuit01"], pose="sohh_standing2"),
    dict(gender=1.0, weight=0.5, race=(0.1, 0.8, 0.1), skin="young_caucasian_male", hair="short03", clothes=["male_casualsuit03"], pose="callharvey3d_standingatattention"),
    dict(gender=0.0, weight=0.5, race=(0.1, 0.8, 0.1), skin="young_caucasian_female", hair="bob02", clothes=["female_casualsuit02"], pose="anrico_standing11"),
]
def tinted(m, hue, sat, val, tag):
    """A copy of a MakeHuman material with its colour shifted (an HSV node before the base colour)."""
    if m is None or not m.use_nodes:
        return m
    m2 = m.copy(); m2.name = f"{m.name}_{tag}"
    nt = m2.node_tree
    for n in nt.nodes:
        if n.type == "BSDF_PRINCIPLED" and n.inputs["Base Color"].is_linked:
            l = n.inputs["Base Color"].links[0]
            hsv = nt.nodes.new("ShaderNodeHueSaturation")
            hsv.inputs["Hue"].default_value = hue; hsv.inputs["Saturation"].default_value = sat; hsv.inputs["Value"].default_value = val
            nt.links.new(l.from_socket, hsv.inputs["Color"]); nt.links.new(hsv.outputs["Color"], n.inputs["Base Color"])
    return m2


SHOES = ["shoes01", "shoes02", "shoes03", "shoes04", "shoes05", "shoes06"]
HAIRV = [(0.5, 1.0, 1.0), (0.5, 0.3, 0.35), (0.5, 0.6, 0.6), (0.52, 1.3, 1.6), (0.46, 1.2, 1.1), (0.5, 0.2, 0.2)]
meshes = []
for i, p in enumerate(CAST):
    r_ = random.Random(i)
    p = dict(p, height=r_.uniform(0.35, 0.7), clothes=p["clothes"] + [SHOES[i % len(SHOES)]])
    objs = human(p, 0, 1.0, 0, 0.0)
    for ob in objs:                    # the clothes and hair recoloured, per person
        if ob.type != "MESH" or re.fullmatch(r"Human(\.\d+)?", ob.name):        # the body keeps its skin
            continue
        hairy = any(h in ob.name for h in ("short0", "long0", "bob0", "afro", "braid", "ponytail"))
        hue, sat, val = r_.choice(HAIRV) if hairy else (r_.random(), r_.uniform(0.25, 0.8), r_.uniform(0.55, 1.3))
        for slot in ob.material_slots:
            slot.material = tinted(slot.material, hue, sat, val, f"c{i}")
    me = kit.frozen(list(objs), f"cast{i}")
    zmin = min(v.co.z for v in me.vertices)
    hx = sum(v.co.x for v in me.vertices) / len(me.vertices); hy = sum(v.co.y for v in me.vertices) / len(me.vertices)
    me.transform(Matrix.Translation(Vector((-hx, -hy, -zmin))))          # its feet on the floor, centred
    meshes.append(me)
tops = [max(v.co.z for v in me.vertices) for me in meshes]
for ob in list(bpy.data.objects):                                       # the rigs left over
    if ob.type == "ARMATURE":
        bpy.data.objects.remove(ob, do_unlink=True)
AISLE = 0.75
rng = random.Random(1995)
crowd = []
while len(crowd) < 130:
    X, d = rng.uniform(-5.0, 5.0), rng.uniform(2.3, 12.6)
    if d < LENS[2] + 0.5 and abs(X - LENS[0] * d / LENS[2]) < 0.45:
        continue
    if math.hypot(X - cx, d - cz) < 0.8:
        continue
    if abs(X) < AISLE:                                              # the aisle down the middle, us to the stage
        continue
    if any(math.hypot(X - x, d - e) < 0.55 for x, e in crowd):
        continue
    crowd.append((X, d))
who = []
for i, (X, d) in enumerate(crowd):
    near = {k for (x, e), k in zip(crowd, who) if math.hypot(X - x, d - e) < 2.5}   # no twins side by side
    # nobody over the screen as we zoom in: in front of it, only those whose heads are under our line to its foot
    line = EY - (EY - SY0) * d / FD - 0.05 if abs(X) - 0.45 < (SX1 + 0.1) * d / FD else 99
    fits = [k for k in range(len(meshes)) if tops[k] < line] or [min(range(len(meshes)), key=tops.__getitem__)]
    k = rng.choice([k for k in fits if k not in near] or fits)
    if tops[k] >= line:
        print("OVER THE SCREEN", X, d, tops[k], line)
    who.append(k)
    me = meshes[k]
    ob = link(bpy.data.objects.new(f"person{i}", me))
    yaw = math.atan2(0 - X, FD - d) + rng.uniform(-0.35, 0.35)     # facing the screen, loosely
    ob.matrix_world = Matrix.Translation(V(X, 0, d)) @ Matrix.Rotation(-yaw, 4, "Z")

# ---- the haze, the lights, the renders ----------------------------------------------------------------------------
haze_m = bpy.data.materials.new("haze"); haze_m.use_nodes = True
nt = haze_m.node_tree
for n in list(nt.nodes):
    if n.type == "BSDF_PRINCIPLED":
        nt.nodes.remove(n)
vol = nt.nodes.new("ShaderNodeVolumePrincipled"); vol.inputs["Density"].default_value = 0.035
nt.links.new(vol.outputs["Volume"], nt.nodes["Material Output"].inputs["Volume"])
haze = box("haze", L_ + 0.05, R_ - 0.05, 0.05, CEIL - 0.05, 0.3, FD - 0.3, haze_m)
scr = bpy.data.objects.new("scr", bpy.data.lights.new("scr", "AREA"))
scr.data.shape = "RECTANGLE"; scr.data.size = SX1 - SX0; scr.data.size_y = SY1 - SY0; scr.data.energy = 3000
scr.location = V(0, (SY0 + SY1) / 2, FD - 0.12); scr.rotation_euler = (math.radians(-90), 0, 0)
link(scr)
# the projector's beam: a spot from the lens on the screen, flattened to its 2:1
pjl = bpy.data.objects.new("pj", bpy.data.lights.new("pj", "SPOT"))
aim = V(0, (SY0 + SY1) / 2, FD) - V(*LENS)
TX, TY = 2.5 / aim.length * 1.01, 1.25 / aim.length * 1.01      # the picture's half-width and half-height, as slopes
pjl.data.spot_size = 2 * math.atan(math.hypot(TX, TY)) * 1.05; pjl.data.spot_blend = 0.0
pjl.data.energy = 1200; pjl.data.shadow_soft_size = 0.02
pjl.matrix_world = Matrix.Translation(V(*LENS)) @ aim.to_track_quat("-Z", "Y").to_matrix().to_4x4()
# a gobo: the light cut to the picture's rectangle (its direction in the light's space against the slopes)
pjl.data.use_nodes = True
lt = pjl.data.node_tree
tc = lt.nodes.new("ShaderNodeTexCoord"); sep = lt.nodes.new("ShaderNodeSeparateXYZ")
lt.links.new(tc.outputs["Normal"], sep.inputs["Vector"])
def slope_ok(axis, t):
    dv = lt.nodes.new("ShaderNodeMath"); dv.operation = "DIVIDE"
    lt.links.new(sep.outputs[axis], dv.inputs[0]); lt.links.new(sep.outputs["Z"], dv.inputs[1])
    ab = lt.nodes.new("ShaderNodeMath"); ab.operation = "ABSOLUTE"; lt.links.new(dv.outputs[0], ab.inputs[0])
    lt_ = lt.nodes.new("ShaderNodeMath"); lt_.operation = "LESS_THAN"; lt_.inputs[1].default_value = t
    lt.links.new(ab.outputs[0], lt_.inputs[0])
    return lt_
mul = lt.nodes.new("ShaderNodeMath"); mul.operation = "MULTIPLY"
lt.links.new(slope_ok("X", TX).outputs[0], mul.inputs[0]); lt.links.new(slope_ok("Y", TY).outputs[0], mul.inputs[1])
em = lt.nodes["Emission"]
lt.links.new(mul.outputs[0], em.inputs["Strength"])
link(pjl)
camera()
cycles(SAMPLES)
sc.cycles.volume_step_rate = 4.0
EMITTERS = [(m, m.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value) for m in GLOWS + [exit_m]]


def lights(mode):
    scr.hide_render = mode != "mon"
    pjl.hide_render = mode != "beam"
    for m, s in EMITTERS:
        m.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = s if mode == "const" else 0.0


MODES = argv[5].split(",") if len(argv) > 5 else ["mon", "beam", "const", "mask"]
for mode in [m_ for m_ in ("mon", "beam", "const") if m_ in MODES]:
    lights(mode)
    render(os.path.join(OUTDIR, f"hall_{mode}.exr"), SCALE)
if "mask" in MODES:
    haze.hide_render = True                # the mask: the object index, no haze, no light worth the name
    lights("none"); sc.cycles.samples = 4; sc.cycles.use_denoising = False
    render(os.path.join(OUTDIR, "hall_mask.exr"), SCALE)
if len(argv) > 4 and argv[4] != "-":
    bpy.ops.wm.save_as_mainfile(filepath=argv[4])
