"""The dorm room in 3D (the flat drawing was 'MSPaint level'): the room, its junk and its people
(MakeHuman via MPFB 2, CC0 assets and poses) built in Blender and rendered ONCE in Cycles through dorm.py's camera.
dorm.py lays the renders out as its layers; nothing here is per frame.

    blender -b -P prototypes/dorm3d/room.py -- OUTDIR TEXDIR [SCALE] [SAMPLES]

Writes OUTDIR/full_{mon,const}.exr (the whole frame at SCALE x 1536 x 768) and OUTDIR/inset_{mon,const}.exr (dorm.py's
INSET at 8x): 'mon' lit by the monitor's picture alone (white, an area light), 'const' by everything else that glows
(the hallway's tubes, the lava lamp, the LEDs), both denoised, with the diffuse colour and the object index (the
picture is index 1). All materials are matte. Room point (X across, Y up, d ahead of the eye) is Blender (X, d, Y);
the eye at 1.45 m, f 1000 px on a 1536-wide frame, the horizon on row 290 of 768.
"""
import math
import os
import sys

import bpy
import bmesh
from mathutils import Matrix, Vector, noise

argv = sys.argv[sys.argv.index("--") + 1:]
OUTDIR, TEX = argv[0], argv[1]
SCALE = int(argv[2]) if len(argv) > 2 else 2
SAMPLES = int(argv[3]) if len(argv) > 3 else 256
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kit import *  # noqa: E402,F403
import kit  # noqa: E402
F, HZ, EY = 1000.0, 290.0, 1.45
MON_X, MON_D = 0.15, 3.85
BD, L_, R_, CEIL = 4.2, -1.3, 1.7, 2.6
DO0, DO1, DH = 2.5, 3.4, 2.05
sc = kit.init(F, HZ, EY)
os.makedirs(OUTDIR, exist_ok=True)


# ---- the room ------------------------------------------------------------------------------------------------------
cinder = mat_brick("cinder", (0.50, 0.55, 0.62), (0.44, 0.48, 0.55), 0.40, 0.20, bump=0.6)       # painted block, pale blue
cinder_h = mat_brick("cinder_h", (0.62, 0.60, 0.50), (0.55, 0.53, 0.44), 0.40, 0.20, bump=0.6)    # the hallway's, cream
tile = mat_brick("tile", (0.30, 0.28, 0.26), (0.22, 0.2, 0.19), 0.30, 0.30, bump=0.1)
ceil_t = mat_brick("ceiltile", (0.62, 0.62, 0.58), (0.4, 0.4, 0.38), 0.61, 0.61, bump=0.2)
rubber = mat("rubber", (0.03, 0.03, 0.035))
# walls: the back, the left, the right around the doorway; floor; ceiling (drop tiles); rubber skirting
box("back", L_, R_, 0, CEIL, BD, BD + 0.1, cinder)
left = box("left", L_ - 0.1, L_, 0, CEIL, 0.3, BD, cinder)
box("right_a", R_, R_ + 0.1, 0, CEIL, 0.3, DO0, cinder)
box("right_b", R_, R_ + 0.1, 0, CEIL, DO1, BD, cinder)
box("right_c", R_, R_ + 0.1, DH, CEIL, DO0, DO1, cinder)
box("floor", L_, R_, -0.05, 0, 0.3, BD, tile)
box("ceiling", L_, R_, CEIL, CEIL + 0.05, 0.3, BD, ceil_t)
for (x0, x1, d0, d1) in ((L_, R_, BD - 0.01, BD), (L_, L_ + 0.01, 0.3, BD), (R_ - 0.01, R_, 0.3, DO0), (R_ - 0.01, R_, DO1, BD)):
    box("skirt", x0, x1, 0, 0.1, d0, d1, rubber)
# the ceiling light (off): a recessed fluorescent troffer
box("troffer", -0.3, 0.3, CEIL - 0.01, CEIL, 2.6, 3.2, mat("troffer", (0.7, 0.7, 0.68)))
box("troffer_lens", -0.27, 0.27, CEIL - 0.012, CEIL - 0.01, 2.63, 3.17, mat_noise("lens", (0.55, 0.56, 0.58), (0.7, 0.7, 0.72), scale=400))

# the doorway: a steel frame, the door open into the room 75 degrees (hinged at DO1), the whiteboard on it
steel = mat("steel", (0.32, 0.3, 0.27))
for (d0, d1, y0, y1) in ((DO0 - 0.06, DO0, 0, DH + 0.06), (DO1, DO1 + 0.06, 0, DH + 0.06), (DO0 - 0.06, DO1 + 0.06, DH, DH + 0.06)):
    box("frame", R_ - 0.03, R_ + 0.12, y0, y1, d0, d1, steel, bevel=0.005)
doorwood = mat_wood("doorwood", (0.30, 0.17, 0.08), (0.42, 0.25, 0.12), scale=3)
ang = math.radians(75)
door_parts = [box("door", 0.0, 0.9, 0, DH - 0.01, -0.022, 0.022, doorwood, bevel=0.004)]
door_parts.append(box("door_board", 0.17, 0.83, 1.07, 1.67, 0.022, 0.032, mat("wb_frame", (0.55, 0.56, 0.58)), bevel=0.003))
door_parts.append(quad("whiteboard", [(0.81, 1.65, 0.0331), (0.19, 1.65, 0.0331), (0.19, 1.09, 0.0331), (0.81, 1.09, 0.0331)],
                       mat_image("whiteboard", os.path.join(TEX, "whiteboard.png"))))
door_parts.append(box("tray", 0.2, 0.8, 1.05, 1.07, 0.022, 0.05, mat("tray", (0.5, 0.5, 0.52))))
door_parts.append(box("marker_b", 0.48, 0.6, 1.07, 1.085, 0.03, 0.045, mat("mk_b", (0.1, 0.2, 0.6))))
door_parts.append(box("marker_r", 0.32, 0.44, 1.07, 1.085, 0.03, 0.045, mat("mk_r", (0.6, 0.05, 0.08))))
kn = mat("knob", (0.55, 0.45, 0.2))
door_parts.append(cyl("knob", 0.82, 0.93, 0.97, 0.05, 0.028, kn))
DM = Matrix(((-math.sin(ang), math.cos(ang), 0, R_ - 0.02), (-math.cos(ang), -math.sin(ang), 0, DO1), (0, 0, 1, 0), (0, 0, 0, 1)))
for p in door_parts:
    # local (along the door from the hinge, out of its room-side face, up) to the room: open 75 degrees into it
    p.matrix_world = DM @ p.matrix_world
# the hallway beyond: floor, the far wall (cream block), ceiling, its tubes (glowing), a door across, a corkboard
HW_ = 1.8
box("hall_floor", R_, R_ + HW_, -0.05, 0, 0.5, 12, tile)
box("hall_far", R_ + HW_, R_ + HW_ + 0.1, 0, CEIL, 0.5, 12, cinder_h)
box("hall_ceiling", R_, R_ + HW_, CEIL, CEIL + 0.05, 0.5, 12, ceil_t)
box("hall_skirt", R_ + HW_ - 0.01, R_ + HW_, 0, 0.1, 0.5, 12, rubber)
box("hall_door", R_ + HW_ - 0.03, R_ + HW_, 0, 2.05, 3.3, 4.2, mat_wood("hdoor", (0.3, 0.18, 0.09), (0.4, 0.25, 0.12), 3))
box("hall_door_sign", R_ + HW_ - 0.035, R_ + HW_ - 0.03, 1.4, 1.55, 3.5, 3.75, mat("sign", (0.8, 0.8, 0.76)))
box("cork", R_ + HW_ - 0.03, R_ + HW_, 1.05, 1.6, 4.6, 5.4, mat_noise("cork", (0.45, 0.3, 0.15), (0.6, 0.42, 0.22), scale=200))
for i, c in enumerate(((0.8, 0.2, 0.2), (0.85, 0.8, 0.3), (0.3, 0.55, 0.8), (0.85, 0.85, 0.85))):
    d0 = 4.68 + i * 0.17
    box("flyer", R_ + HW_ - 0.035, R_ + HW_ - 0.03, 1.32 - (i % 2) * 0.1, 1.5 - (i % 2) * 0.1, d0, d0 + 0.12, mat(f"flyer{i}", c))
TUBE = mat("tube", (1, 1, 1), emit=(1.0, 0.97, 0.9), strength=6.0)
tubes = []
for d0 in (2.2, 4.4, 6.6):
    tubes.append(box("tube", R_ + 0.7, R_ + 1.1, CEIL - 0.03, CEIL, d0, d0 + 1.0, TUBE))

# the posters: the ray-traced one on the right wall; a band poster on the left wall over the bunk
PD0, PD1 = 3.55, 4.1
quad("poster", [(L_ + 0.003, 2.45, 2.45), (L_ + 0.003, 2.45, 2.95), (L_ + 0.003, 1.7, 2.95), (L_ + 0.003, 1.7, 2.45)],
     mat_image("poster", os.path.join(TEX, "poster.png")))

# the bunk bed along the left wall: wooden posts and rails, slats, mattresses, rumpled blankets, pillows, the ladder
BX0, BX1, BN = L_, -0.4, 2.2
pine = mat_wood("pine", (0.42, 0.27, 0.13), (0.55, 0.37, 0.19), scale=4)
for (x, d) in ((BX1 - 0.07, BN), (BX1 - 0.07, BD - 0.07), (BX0 + 0.01, BN), (BX0 + 0.01, BD - 0.07)):
    box("post", x, x + 0.07, 0, 1.95, d, d + 0.07, pine, bevel=0.008)
LOW, HIGH = 0.5, 1.45                  # the mattresses' tops
for (top, blanket) in ((LOW, (0.10, 0.17, 0.35)), (HIGH, (0.40, 0.08, 0.10))):
    y0 = top - 0.32
    box("rail_side", BX1 - 0.07, BX1, y0, y0 + 0.16, BN, BD, pine, bevel=0.006)
    box("rail_end", BX0, BX1, y0, y0 + 0.16, BN, BN + 0.07, pine, bevel=0.006)
    box("mattress", BX0 + 0.04, BX1 - 0.08, y0 + 0.14, top - 0.04, BN + 0.08, BD - 0.02,
        mat_noise("ticking", (0.75, 0.75, 0.72), (0.65, 0.66, 0.66), scale=80), bevel=0.04, seg=4)
box("guard", BX1 - 0.07, BX1, HIGH + 0.1, HIGH + 0.25, BN + 0.5, BD - 0.07, pine, bevel=0.006)
box("guard_end", BX0, BX1, HIGH + 0.12, HIGH + 0.25, BN, BN + 0.07, pine, bevel=0.006)
for i in range(5):                     # the ladder: rungs between the front post and a rail at the foot
    y = 0.42 + i * 0.27
    box("rung", BX1 - 0.06, BX1 - 0.02, y, y + 0.035, BN + 0.07, BN + 0.42, pine, bevel=0.005)
box("ladder_rail", BX1 - 0.07, BX1, 0, HIGH + 0.25, BN + 0.42, BN + 0.48, pine, bevel=0.006)

# cloth: the blankets and the laundry (a third of the first heap) laid one at a time over a heightfield of the
# bunk (each settles on what's under it, crumpled by noise, hanging over the open side), then smoothed
import random
import numpy as np
rnd = random.Random(11)
HX0, HX1, HD0, HD1, HRES = BX0, BX1, BN, BD, 0.01
NX, ND = int((HX1 - HX0) / HRES) + 1, int((HD1 - HD0) / HRES) + 1


def heights(top):
    hf = np.zeros((NX, ND))
    xs = HX0 + np.arange(NX) * HRES
    ds = HD0 + np.arange(ND) * HRES
    inside = ((xs > BX0 + 0.04) & (xs < BX1 - 0.08))[:, None] & ((ds > BN + 0.08) & (ds < BD - 0.02))[None, :]
    hf[inside] = top - 0.04
    return hf


def hget(hf, x, d):
    i = min(NX - 1, max(0, int(round((x - HX0) / HRES)))); j = min(ND - 1, max(0, int(round((d - HD0) / HRES))))
    return hf[i, j]


def drape(name, hf, cx, cd, sx, sd, rot, m, crumple=0.02, shape="sheet", seed=0, n=40):
    """A garment laid on the heightfield hf (updated): a grid cut to its shape, settled, crumpled."""
    off = Vector((seed * 3.1, seed * 1.7, 0))
    keep = {}
    vs = []
    for a in range(n + 1):
        for b_ in range(n + 1):
            u, v = a / n * 2 - 1, b_ / n * 2 - 1            # u across the garment, v along it
            if shape == "tee" and not ((abs(u) < 0.55 and v < 0.75) or (v > 0.35 and v < 0.75) or (abs(u) < 0.2 and v < 0.85)):
                continue
            if shape == "jeans" and not (v > 0.55 or (0.12 < abs(u) < 0.95 and abs(u) < 0.95 - (0.55 - v) * 0.05)):
                continue
            x = cx + (u * sx * math.cos(rot) - v * sd * math.sin(rot)) / 2
            d = cd + (u * sx * math.sin(rot) + v * sd * math.cos(rot)) / 2
            base = hget(hf, x, d)
            if x < BX0 + 0.05:                             # bunched against the wall
                x = BX0 + 0.05; base = hget(hf, x, d)
            if base <= 0:                                  # off the mattress's open side or ends: hang down
                over = max(0, x - (BX1 - 0.08)) + max(0, BN + 0.08 - d) + max(0, d - BD + 0.02)
                base = max(0.05, hget(hf, min(x, BX1 - 0.09), min(max(d, BN + 0.09), BD - 0.03)) - over * 1.0)
            z = base + 0.008 + crumple * (1 + noise.noise(Vector((u * 2.2, v * 2.2, 0)) + off)) + crumple * 0.4 * noise.noise(Vector((u * 7, v * 7, 1)) + off)
            keep[(a, b_)] = len(vs)
            vs.append(V(x, z, d))
    fs = []
    for a in range(n):
        for b_ in range(n):
            q_ = [(a, b_), (a + 1, b_), (a + 1, b_ + 1), (a, b_ + 1)]
            if all(k in keep for k in q_):
                fs.append(tuple(keep[k] for k in q_))
    ob = mesh_ob(name, vs, fs, m)
    for p in ob.data.polygons:
        p.use_smooth = True
    for vco in vs:                                         # what the next one lands on
        x, d, z = vco
        i = int(round((x - HX0) / HRES)); j = int(round((d - HD0) / HRES))
        if 0 <= i < NX and 0 <= j < ND:
            sl = (slice(max(0, i - 1), i + 2), slice(max(0, j - 1), j + 2))
            hf[sl] = np.maximum(hf[sl], z + 0.004)
    ss = ob.modifiers.new("ss", "SUBSURF"); ss.levels = 1; ss.render_levels = 1
    so = ob.modifiers.new("thick", "SOLIDIFY"); so.thickness = 0.005
    return ob


hf_lo, hf_hi = heights(LOW), heights(HIGH)
drape("blanket_low", hf_lo, (BX0 + BX1) / 2 - 0.02, 3.2, 1.0, 1.85, 0.03,
      mat_noise("blanket_lo", (0.08, 0.14, 0.32), (0.13, 0.2, 0.42), scale=150, bump=0.2), crumple=0.012, seed=1)
drape("blanket_high", hf_hi, (BX0 + BX1) / 2 - 0.02, 3.2, 1.0, 1.85, -0.03,
      mat_noise("blanket_hi", (0.35, 0.06, 0.08), (0.45, 0.1, 0.12), scale=150, bump=0.2), crumple=0.015, seed=2)
LCOL = [((0.10, 0.15, 0.35), (0.14, 0.2, 0.45), "jeans"), ((0.35, 0.35, 0.37), (0.45, 0.45, 0.47), "tee"),
        ((0.8, 0.8, 0.76), (0.9, 0.9, 0.86), "tee"), ((0.5, 0.06, 0.08), (0.6, 0.1, 0.1), "sheet"),
        ((0.04, 0.04, 0.05), (0.08, 0.08, 0.09), "tee"), ((0.1, 0.3, 0.14), (0.14, 0.38, 0.18), "sheet"),
        ((0.6, 0.45, 0.12), (0.7, 0.55, 0.2), "tee"), ((0.08, 0.12, 0.3), (0.12, 0.17, 0.4), "jeans")]
for i, (c0, c1, shape) in enumerate(LCOL):
    big = shape == "jeans"
    drape(f"laundry{i}", hf_lo, -0.85 + rnd.uniform(-0.15, 0.15), 3.3 + rnd.uniform(-0.3, 0.3),
          0.55 if big else rnd.uniform(0.45, 0.6), 1.0 if big else rnd.uniform(0.55, 0.7), rnd.uniform(0, 6.28),
          mat_noise(f"fabric{i}", c0, c1, scale=300, bump=0.15), crumple=rnd.uniform(0.025, 0.045), shape=shape, seed=10 + i)
# pillows at the bunks' heads (the back)
pillow = mat_noise("pillow", (0.82, 0.82, 0.78), (0.9, 0.9, 0.86), scale=200)
blob("pillow_lo", -0.85, LOW + 0.07, BD - 0.3, 0.3, 0.07, 0.18, pillow, seed=1, rough=0.12)
blob("pillow_hi", -0.85, HIGH + 0.07, BD - 0.3, 0.3, 0.07, 0.18, pillow, seed=2, rough=0.12)

# the desk against the back wall, a drawer, its legs
DX0, DX1, DD = -0.3, 0.8, BD - 0.55
oak = mat_wood("oak", (0.35, 0.2, 0.09), (0.48, 0.3, 0.14), scale=5)
box("desk_top", DX0, DX1, 0.71, 0.745, DD, BD, oak, bevel=0.004)
box("desk_apron", DX0 + 0.02, DX1 - 0.02, 0.62, 0.71, DD + 0.01, DD + 0.03, oak, bevel=0.003)
box("drawer_pull", 0.2, 0.32, 0.655, 0.67, DD - 0.005, DD + 0.01, steel)
for x in (DX0 + 0.02, DX1 - 0.06):
    for d in (DD + 0.02, BD - 0.06):
        box("desk_leg", x, x + 0.04, 0, 0.62, d, d + 0.04, oak)

# the CRT: a beige case, the picture's bezel, its tapered back, the foot, badge, LED, buttons; the picture (index 1)
beige = mat_noise("beige", (0.62, 0.58, 0.48), (0.66, 0.62, 0.52), scale=500)
beige_dk = mat_noise("beige_dk", (0.48, 0.45, 0.37), (0.52, 0.49, 0.41), scale=500)
mx0, mx1 = MON_X - 0.21, MON_X + 0.21
for (x0, x1, y0, y1) in ((mx0, mx1, 1.09, 1.14), (mx0, mx1, 0.78, 0.8), (mx0, MON_X - 0.18, 0.8, 1.09), (MON_X + 0.18, mx1, 0.8, 1.09)):
    box("crt_face", x0, x1, y0, y1, MON_D - 0.022, MON_D + 0.04, beige, bevel=0.006, seg=2)
box("crt_tube", MON_X - 0.18, MON_X + 0.18, 0.8, 1.09, MON_D + 0.0, MON_D + 0.04, beige_dk)
vs = [V(mx0 + 0.01, 1.13, MON_D + 0.04), V(mx1 - 0.01, 1.13, MON_D + 0.04), V(mx1 - 0.01, 0.79, MON_D + 0.04), V(mx0 + 0.01, 0.79, MON_D + 0.04),
      V(mx0 + 0.09, 1.11, MON_D + 0.34), V(mx1 - 0.09, 1.11, MON_D + 0.34), V(mx1 - 0.09, 0.81, MON_D + 0.34), V(mx0 + 0.09, 0.81, MON_D + 0.34)]
crt_back = mesh_ob("crt_back", vs, [(0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7), (4, 5, 6, 7)], beige)
bv = crt_back.modifiers.new("bv", "BEVEL"); bv.width = 0.01; bv.segments = 3
for i in range(6):                     # vents on the top
    box("vent", mx0 + 0.1, mx1 - 0.1, 1.112 + 0.004 * (i / 6), 1.118, MON_D + 0.1 + i * 0.03, MON_D + 0.11 + i * 0.03, beige_dk)
# the bezel's slope into the picture, the picture itself (flat, exactly dorm.py's SCREEN), black glass when off
bz = [(MON_X - 0.18, 1.09), (MON_X + 0.18, 1.09), (MON_X + 0.18, 0.8), (MON_X - 0.18, 0.8)]
pic = [(MON_X - 0.16, 1.06), (MON_X + 0.16, 1.06), (MON_X + 0.16, 0.82), (MON_X - 0.16, 0.82)]
vs = [V(x, y, MON_D - 0.021) for x, y in bz] + [V(x, y, MON_D - 0.005) for x, y in pic]
mesh_ob("bezel", vs, [(0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)], beige_dk)
screen = mesh_ob("picture", [V(x, y, MON_D - 0.005) for x, y in pic], [(3, 2, 1, 0)], mat("glass", (0.02, 0.025, 0.03)))
screen.pass_index = 1
box("crt_foot", MON_X - 0.12, MON_X + 0.12, 0.745, 0.78, MON_D + 0.02, MON_D + 0.28, beige_dk, bevel=0.01)
box("badge", MON_X - 0.15, MON_X - 0.07, 0.785, 0.795, MON_D - 0.022, MON_D - 0.02, mat("badge", (0.3, 0.3, 0.32)))
for i in range(3):
    box("button", MON_X + 0.06 + i * 0.025, MON_X + 0.075 + i * 0.025, 0.784, 0.794, MON_D - 0.024, MON_D - 0.02, beige_dk)
LED_G = mat("led_g", (0.1, 1, 0.2), emit=(0.1, 1, 0.2), strength=8.0)
LED_R = mat("led_r", (1, 0.15, 0.1), emit=(1, 0.15, 0.1), strength=8.0)
leds = [box("led", MON_X + 0.165, MON_X + 0.175, 0.786, 0.794, MON_D - 0.024, MON_D - 0.02, LED_G)]

# the keyboard (keys in rows), the mouse and its pad, a cable
kb = mat_noise("kb", (0.6, 0.57, 0.48), (0.64, 0.6, 0.5), scale=500)
key = mat("key", (0.66, 0.63, 0.54))
box("keyboard", MON_X - 0.23, MON_X + 0.23, 0.745, 0.765, DD + 0.04, DD + 0.21, kb, bevel=0.004)
for r in range(5):
    for c in range(18):
        x = MON_X - 0.215 + c * 0.0235
        box("key", x, x + 0.019, 0.765, 0.775 + r * 0.002, DD + 0.055 + r * 0.03, DD + 0.077 + r * 0.03, key, bevel=0.002, seg=1)
box("mousepad", MON_X + 0.29, MON_X + 0.5, 0.745, 0.748, DD + 0.04, DD + 0.24, mat("pad", (0.1, 0.12, 0.3)))
blob("mouse", MON_X + 0.38, 0.758, DD + 0.13, 0.03, 0.015, 0.05, kb, rough=0.02)

# a soda can by the keyboard; the lava lamp left of the monitor (glass, glowing wax, a base and cap)
red_can = mat_noise("can", (0.6, 0.04, 0.05), (0.75, 0.08, 0.08), scale=20)
alu = mat("alu", (0.6, 0.6, 0.62))
cyl("can", -0.09, 0.745, 0.866, DD + 0.15, 0.033, red_can)
cyl("can_top", -0.09, 0.866, 0.872, DD + 0.15, 0.03, alu)
lamp_metal = mat("lampmetal", (0.12, 0.12, 0.13))
LX, LD = -0.22, BD - 0.2
cyl("lamp_base", LX, 0.745, 0.83, LD, 0.06, lamp_metal, r1=0.035)
cyl("lamp_glass", LX, 0.83, 1.05, LD, 0.035, mat("lavaliquid", (0.4, 0.12, 0.03), emit=(1.0, 0.35, 0.05), strength=1.5), r1=0.022)
cyl("lamp_cap", LX, 1.05, 1.1, LD, 0.022, lamp_metal, r1=0.012)
WAX = mat("wax", (1, 0.5, 0.1), emit=(1.0, 0.45, 0.08), strength=6.0)
wax = [blob("wax", LX + 0.002, 0.88, LD, 0.022, 0.03, 0.022, WAX, seed=4, rough=0.15),
       blob("wax", LX - 0.004, 0.98, LD, 0.014, 0.02, 0.014, WAX, seed=5, rough=0.2),
       blob("wax", LX, 0.845, LD, 0.03, 0.012, 0.03, WAX, seed=6, rough=0.1)]
lava = bpy.data.objects.new("lava_light", bpy.data.lights.new("lava_light", "POINT"))
lava.data.energy = 6; lava.data.color = (1.0, 0.45, 0.1); lava.data.shadow_soft_size = 0.03; lava.location = V(LX, 0.93, LD); link(lava)

# the pizza box on the desk's right end: flat, turned 15 degrees as if thrown there, its lid open back on the
# wall; the pizza inside with slices gone
card = mat_noise("cardboard", (0.5, 0.38, 0.22), (0.58, 0.45, 0.28), scale=60)
card_in = mat_noise("cardboard_in", (0.62, 0.52, 0.36), (0.68, 0.58, 0.42), scale=60)
pcx, pcd = 0.58, BD - 0.29
pz_parts = []
pz_parts.append(box("pz_bottom", -0.2, 0.2, 0.745, 0.75, -0.2, 0.2, card_in))
for (x0, x1, d0, d1, top) in ((-0.2, 0.2, -0.2, -0.195, 0.762), (-0.2, -0.195, -0.2, 0.2, 0.785), (0.195, 0.2, -0.2, 0.2, 0.785), (-0.2, 0.2, 0.195, 0.2, 0.79)):
    pz_parts.append(box("pz_wall", x0, x1, 0.745, top, d0, d1, card))
pz = quad("pizza", [(-0.18, 0.758, 0.18), (0.18, 0.758, 0.18), (0.18, 0.758, -0.18), (-0.18, 0.758, -0.18)],
          mat_image("pizza", os.path.join(TEX, "pizza.png"), alpha=True))
pz_parts.append(pz)
lid = box("pz_lid", -0.2, 0.2, 0.0, 0.005, 0.0, 0.4, card)
lid_in = quad("pz_lid_in", [(-0.19, -0.001, 0.39), (0.19, -0.001, 0.39), (0.19, -0.001, 0.01), (-0.19, -0.001, 0.01)], card_in)
for p in (lid, lid_in):                # hinged on the box's back edge, leaning back on the wall (about 70 degrees)
    p.matrix_world = Matrix.Translation(V(0, 0.79, 0.2)) @ Matrix.Rotation(math.radians(80), 4, "X") @ p.matrix_world
    pz_parts.append(p)
for p in pz_parts:
    p.matrix_world = Matrix.Translation(V(pcx, 0, pcd)) @ Matrix.Rotation(math.radians(15), 4, "Z") @ p.matrix_world

# the bookshelf over the desk: two boards on brackets, books (the fat black ABRASH among them), junk on top
SX0, SX1, SDP = -0.38, 0.88, 0.22
shelfwood = mat_wood("shelf", (0.4, 0.26, 0.13), (0.5, 0.34, 0.18), scale=5)
for sy in (1.40, 1.76):
    box("board", SX0, SX1, sy - 0.025, sy, BD - SDP, BD, shelfwood, bevel=0.003)
    for x in (SX0 + 0.06, SX1 - 0.08):
        box("bracket", x, x + 0.02, sy - 0.18, sy - 0.025, BD - 0.17, BD, steel)
rb = random.Random(7)
BOOKC = [(0.45, 0.06, 0.07), (0.08, 0.16, 0.38), (0.6, 0.5, 0.12), (0.08, 0.28, 0.13), (0.7, 0.7, 0.65), None,
         (0.28, 0.1, 0.32), (0.7, 0.3, 0.06), (0.12, 0.12, 0.13), (0.5, 0.09, 0.15)]
WIDTHS = [0.035, 0.03, 0.05, 0.028, 0.04, 0.09, 0.032, 0.045, 0.03, 0.038]
HTS = [0.24, 0.22, 0.26, 0.2, 0.25, 0.27, 0.23, 0.21, 0.24, 0.22]
x = SX0 + 0.12
spine = mat_image("abrash", os.path.join(TEX, "abrash_spine.png"))
black = mat("blackbook", (0.015, 0.015, 0.018))
for i, (wd, ht, col) in enumerate(zip(WIDTHS, HTS, BOOKC)):
    d0, d1 = BD - 0.2 + rb.uniform(0, 0.03), BD - 0.02
    if col is None:                    # ABRASH: the fat black book, its spine readable
        box("abrash", x, x + wd, 1.40, 1.40 + ht, d0, d1, black, bevel=0.003)
        quad("abrash_spine", [(x + 0.003, 1.40 + ht - 0.004, d0 - 0.0005), (x + wd - 0.003, 1.40 + ht - 0.004, d0 - 0.0005),
                              (x + wd - 0.003, 1.404, d0 - 0.0005), (x + 0.003, 1.404, d0 - 0.0005)], spine)
    else:
        m_ = mat_noise(f"book{i}", col, tuple(min(1, c * 1.25) for c in col), scale=30)
        box("book", x, x + wd, 1.40, 1.40 + ht, d0, d1, m_, bevel=0.003)
        box("book_band", x - 0.0005, x + wd + 0.0005, 1.40 + ht * 0.78, 1.40 + ht * 0.82, d0 - 0.0005, d0 + 0.002, mat(f"band{i}", (0.75, 0.65, 0.3)))
    x += wd + 0.004
lean = box("book_lean", 0, 0.03, 0, 0.22, -0.09, 0.09, mat_noise("bookl", (0.2, 0.4, 0.5), (0.25, 0.5, 0.6), scale=30), bevel=0.003)
rot_about(lean, Vector((0, 0, 0)), math.radians(-24), "Y"); lean.location = V(x + 0.02, 1.40, BD - 0.11)
x += 0.16
for i, (th, col) in enumerate(((0.035, (0.5, 0.15, 0.08)), (0.03, (0.12, 0.12, 0.35)), (0.04, (0.6, 0.57, 0.45)))):
    y0 = 1.40 + sum((0.035, 0.03, 0.04)[:i])
    b_ = box("book_flat", -0.11, 0.11, y0, y0 + th, -0.08, 0.08, mat_noise(f"bookf{i}", col, tuple(min(1, c * 1.2) for c in col), 30), bevel=0.003)
    rot_about(b_, Vector((0, 0, 0)), math.radians(rb.uniform(-8, 8)), "Z"); b_.location = V(x + 0.1, 0, BD - 0.11)
x = SX0 + 0.05
for i in range(9):                     # the upper board: more books
    wd, ht = rb.uniform(0.025, 0.055), rb.uniform(0.17, 0.26)
    col = (rb.uniform(0.05, 0.6), rb.uniform(0.05, 0.5), rb.uniform(0.05, 0.5))
    box("book", x, x + wd, 1.76, 1.76 + ht, BD - 0.2, BD - 0.02, mat_noise(f"bk2{i}", col, tuple(min(1, c * 1.25) for c in col), 30), bevel=0.003)
    x += wd + 0.004
mug = mat("mug", (0.75, 0.72, 0.65))
cyl("mug", x + 0.07, 1.76, 1.86, BD - 0.11, 0.04, mug)
tor = bpy.data.objects.new("mug_handle", bpy.data.meshes.new("mh")); link(tor)
bm = bmesh.new(); bmesh.ops.create_cone(bm, cap_ends=False, segments=12, radius1=0.004, radius2=0.004, depth=0.01); bm.free()
blob("mug_handle", x + 0.115, 1.81, BD - 0.11, 0.012, 0.03, 0.006, mug, rough=0.0)
x += 0.17
for i, c in enumerate(((0.05, 0.05, 0.06), (0.5, 0.08, 0.08), (0.08, 0.15, 0.45), (0.05, 0.05, 0.06), (0.6, 0.6, 0.15))):
    f_ = box("floppy", -0.045, 0.045, 1.76 + 0.0035 * i, 1.76 + 0.0035 * (i + 1), -0.047, 0.047, mat(f"floppy{i}", c))
    rot_about(f_, Vector((0, 0, 0)), math.radians(rb.uniform(-12, 12)), "Z"); f_.location = V(x + 0.05, 0, BD - 0.11)
x += 0.14
gold = mat("gold", (0.6, 0.45, 0.1))
box("trophy_base", x + 0.02, x + 0.08, 1.76, 1.79, BD - 0.14, BD - 0.08, mat("trophy_base", (0.15, 0.1, 0.06)))
cyl("trophy_stem", x + 0.05, 1.79, 1.85, BD - 0.11, 0.008, gold)
cyl("trophy_cup", x + 0.05, 1.85, 1.91, BD - 0.11, 0.012, gold, r1=0.03)
x += 0.12
blue_can = mat_noise("bcan", (0.06, 0.16, 0.45), (0.09, 0.22, 0.55), scale=20)
for row, n in enumerate((3, 2, 1)):    # a can pyramid
    for j in range(n):
        cyl("pyr_can", x + 0.035 + j * 0.068 + row * 0.034, 1.76 + 0.122 * row, 1.76 + 0.122 * (row + 1), BD - 0.11, 0.033,
            red_can if (row + j) % 2 == 0 else blue_can)

# the tower on the floor under the desk's right end: beige case, drive bays, the floppy slot, LEDs; the socks
tx0, tx1, TF, TB, TT = 0.5, 0.7, BD - 0.48, BD - 0.04, 0.44
box("tower", tx0, tx1, 0, TT, TF, TB, beige, bevel=0.008)
for i in range(3):
    y = TT - 0.06 - i * 0.05
    box("bay", tx0 + 0.02, tx1 - 0.02, y - 0.035, y, TF - 0.004, TF + 0.01, beige_dk, bevel=0.002)
box("floppy_slot", tx0 + 0.05, tx1 - 0.05, TT - 0.25, TT - 0.243, TF - 0.005, TF + 0.01, mat("slot", (0.02, 0.02, 0.02)))
for (x_, m_) in ((tx0 + 0.03, LED_R), (tx0 + 0.06, LED_G)):
    leds.append(box("led", x_ - 0.005, x_ + 0.005, TT - 0.305, TT - 0.295, TF - 0.006, TF, m_))
box("power", tx1 - 0.06, tx1 - 0.03, TT - 0.32, TT - 0.29, TF - 0.006, TF, beige_dk, bevel=0.003)
# a pair of purple socks: one over the front edge hanging down, heel and toe; one
# over the left side
sock = mat_noise("sock", (0.30, 0.10, 0.48), (0.38, 0.15, 0.58), scale=400, bump=0.3)
sock_dk = mat_noise("sock_dk", (0.18, 0.06, 0.3), (0.24, 0.09, 0.38), scale=400, bump=0.3)
tube("sock1", [(0.58, TT + 0.012, 3.98), (0.585, TT + 0.012, 3.84), (0.59, TT + 0.006, TF + 0.01), (0.593, TT - 0.04, TF - 0.012),
               (0.596, TT - 0.14, TF - 0.016), (0.6, TT - 0.19, TF - 0.03), (0.62, TT - 0.215, TF - 0.075), (0.64, TT - 0.22, TF - 0.11)],
     0.024, sock, taper=[1.0, 1.0, 0.95, 0.9, 0.9, 1.0, 0.95, 0.6])
blob("sock1_heel", 0.6, TT - 0.19, TF - 0.022, 0.024, 0.024, 0.02, sock_dk, rough=0.05)
blob("sock1_toe", 0.638, TT - 0.219, TF - 0.105, 0.02, 0.016, 0.024, sock_dk, rough=0.05)
tube("sock2", [(0.66, TT + 0.012, 4.06), (0.6, TT + 0.012, 3.98), (0.53, TT + 0.012, 3.95), (tx0 + 0.004, TT + 0.004, 3.94),
               (tx0 - 0.015, TT - 0.05, 3.935), (tx0 - 0.018, TT - 0.12, 3.93), (tx0 - 0.03, TT - 0.165, 3.9), (tx0 - 0.045, TT - 0.18, 3.85)],
     0.023, sock, taper=[0.9, 1.0, 1.0, 0.95, 0.9, 1.0, 0.95, 0.6])
blob("sock2_toe", tx0 - 0.044, TT - 0.18, 3.855, 0.02, 0.016, 0.024, sock_dk, rough=0.05)

# the wire wastepaper basket by the desk's left leg, overflowing: napkins, a crushed can, a crust; one that missed
WX, WD, WR, WHT = -0.36, BD - 0.38, 0.12, 0.3
bask = cyl("basket", WX, 0, WHT, WD, WR * 0.8, mat("wire", (0.2, 0.2, 0.22)), n=32, r1=WR, cap=False)
bpy.context.view_layer.objects.active = bask
sd = bask.modifiers.new("sub", "SUBDIVIDE") if False else None
bm = bmesh.new(); bm.from_mesh(bask.data)
bmesh.ops.subdivide_edges(bm, edges=[e for e in bm.edges if abs(e.verts[0].co.z - e.verts[1].co.z) > 0.01], cuts=8)
bm.to_mesh(bask.data); bm.free()
wf = bask.modifiers.new("wire", "WIREFRAME"); wf.thickness = 0.004
cyl("basket_rim", WX, WHT - 0.008, WHT, WD, WR + 0.003, mat("wire2", (0.25, 0.25, 0.27)), n=32, cap=False).modifiers.new("s", "SOLIDIFY").thickness = 0.006
napkin = mat_noise("napkin", (0.85, 0.84, 0.8), (0.92, 0.92, 0.9), scale=300, bump=0.4)
for i in range(13):
    a = rnd.uniform(0, 2 * math.pi); rr = rnd.uniform(0, WR * 0.9)
    blob("napkin", WX + rr * math.cos(a), WHT - 0.02 + rnd.uniform(0, 0.1) + (0.06 if rr < 0.05 else 0), WD + rr * math.sin(a),
         rnd.uniform(0.03, 0.05), rnd.uniform(0.025, 0.04), rnd.uniform(0.03, 0.05), napkin, seed=20 + i, rough=0.5, subdiv=3)
cc = cyl("crushed_can", WX + 0.05, WHT + 0.04, WHT + 0.1, WD - 0.04, 0.03, red_can, r1=0.026)
rot_about(cc, V(WX + 0.05, WHT + 0.07, WD - 0.04), 0.9, "Y")
blob("crust", WX - 0.06, WHT + 0.07, WD - 0.05, 0.06, 0.015, 0.02, mat_noise("crust", (0.45, 0.25, 0.08), (0.6, 0.38, 0.14), 80), seed=33, rough=0.2)
blob("napkin_floor", WX + 0.2, 0.025, WD - 0.12, 0.04, 0.025, 0.04, napkin, seed=40, rough=0.5)

# the couch in the foreground, its back to us: dark red fabric, a rounded top, seat cushions (the people sit on them)
couchf = mat_noise("couch", (0.16, 0.025, 0.03), (0.22, 0.04, 0.045), scale=500, bump=0.25)
box("couch_back", -1.65, 1.65, 0.05, 0.8, 1.85, 2.02, couchf, bevel=0.04, seg=4)
cb = cyl("couch_roll", 0, 0.0, 1.0, 0, 0.075, couchf, n=24)
cb.matrix_world = Matrix.Translation(V(-1.65, 0.79, 1.94)) @ Matrix.Rotation(math.radians(-90), 4, "Y") @ Matrix.Scale(3.3, 4, Vector((0, 0, 1)))
# (the roll: a cylinder along X; scaled along its own axis to the couch's width)
for i, x0 in enumerate((-1.5, -0.5, 0.5)):
    blob("cushion", x0 + 0.5, 0.47, 2.35, 0.48, 0.07, 0.3, couchf, seed=50 + i, rough=0.05)
box("couch_base", -1.65, 1.65, 0.1, 0.4, 1.9, 2.7, couchf, bevel=0.03)

# ---- the people (MakeHuman: MPFB 2 and its CC0 system assets, CC0 poses) ------------------------------------------
PEOPLE = [
    dict(X=-0.98, d=2.14, z=0.56, gender=0.0, weight=0.45, race=(0.1, 0.8, 0.1), skin="young_caucasian_female",
         hair="long01", clothes=["female_casualsuit01"], pose="callharvey3d_sittingnatural"),
    dict(X=-0.48, d=2.14, z=0.56, gender=1.0, weight=0.6, race=(0.6, 0.3, 0.1), skin="young_asian_male",
         hair="short02", clothes=["male_casualsuit03"], pose="anrico_sitting10"),
    dict(X=0.62, d=2.18, z=0.56, gender=1.0, weight=0.5, race=(0.1, 0.8, 0.1), skin="young_caucasian_male",
         hair="short04", clothes=["male_casualsuit01"], pose="mindfront_sitting_in_armchair_holding_wine_glass"),
    dict(X=1.1, d=2.14, z=0.56, gender=0.0, weight=0.5, race=(0.0, 0.1, 0.9), skin="young_african_female",
         hair="afro01", clothes=["female_sportsuit01"], pose="callharvey3d_sittingdefault"),
    dict(X=-0.9, d=3.5, z=HIGH + 0.08, gender=1.0, weight=0.5, race=(0.2, 0.7, 0.1), skin="young_caucasian_male2",
         hair="short01", clothes=["male_casualsuit05"], pose="callharvey3d_lotus"),       # the top bunk, cross-legged
]
for p in PEOPLE:
    human(p, p["X"], p["z"], p["d"], math.atan2(MON_X - p["X"], MON_D - p["d"]) * 0.5)

# ---- lights, camera, the renders ----------------------------------------------------------------------------------
mon = bpy.data.objects.new("mon", bpy.data.lights.new("mon", "AREA"))
mon.data.shape = "RECTANGLE"; mon.data.size = 0.32; mon.data.size_y = 0.24; mon.data.energy = 20
mon.location = V(MON_X, 0.94, MON_D - 0.03); mon.rotation_euler = (math.radians(-90), 0, 0)
link(mon)
hall_lights = []
for d0 in (2.2, 4.4, 6.6):
    t = bpy.data.objects.new("tube_l", bpy.data.lights.new("tube_l", "AREA"))
    t.data.shape = "RECTANGLE"; t.data.size = 0.4; t.data.size_y = 1.0; t.data.energy = 35
    t.data.color = (1.0, 0.97, 0.9); t.location = V(R_ + 0.9, CEIL - 0.04, d0 + 0.5)
    link(t); hall_lights.append(t)

camera()
cycles(SAMPLES)
EMITTERS = [(m, m.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value) for m in (TUBE, LED_G, LED_R, WAX, bpy.data.materials["lavaliquid"])]


def lights(mode):
    mon.hide_render = mode != "mon"
    lava.hide_render = mode == "mon"
    for t in hall_lights:
        t.hide_render = mode == "mon"
    for m, s in EMITTERS:
        m.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = 0.0 if mode == "mon" else s


_a, _b = P(MON_X - 0.16, 1.06, MON_D), P(MON_X + 0.16, 0.82, MON_D)
INSET = (_a[0] - 140, _a[1] - 90, _b[0] + 140, _b[1] + 90)
only = argv[4] if len(argv) > 4 else "all"
for mode in ("mon", "const"):
    lights(mode)
    if only in ("all", "full"):
        render(os.path.join(OUTDIR, f"full_{mode}.exr"), SCALE)
    if only in ("all", "inset"):
        render(os.path.join(OUTDIR, f"inset_{mode}.exr"), 8, INSET)
if len(argv) > 5:
    bpy.ops.wm.save_as_mainfile(filepath=argv[5])
