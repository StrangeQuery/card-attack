"""Bake the dancers: Quaternius's CC0 characters (Poly Pizza) posed by procedural dance moves, one mesh per phase.

    blender -b --factory-startup -P pose_people.py -- DLDIR OUT.npz [MODEL ...]

The beat is 120 BPM at 12 fps: 6 frames a beat; a move is a 2-beat cycle of 12 phases. Each phase's pose is bone
rotations from the rest pose (a T-pose; angles in degrees about the bones' own axes, found by test: UpperArm.R X+
lifts the arm up out of the T (on both sides), Z+ swings it forward (the .L side's Y and Z mirrored);
LowerArm the same about the elbow; Torso/Head X+ bow forward, Y turns, Z leans to the character's right) plus a lift of the whole body (metres, after scaling
to the character's height). The sitting moves start from the model's own Sitting clip and override the arms and
head. Out: per model, per move, per phase, the posed vertices (Blender's Z up, the character facing -Y) and the
shared triangles, the hands' places (for a phone, a can, a lighter) and the lift.
"""
import math
import sys

import bpy
import numpy as np
from mathutils import Euler

args = sys.argv[sys.argv.index("--") + 1:]
DL, OUT = args[0], args[1]
MODELS = {"man_a": "HMnuH5geEG", "man_b": "fjHyMd5Wxw", "man_c": "DLptRuewTn", "woman_a": "jpKRgGDxhk",
          "woman_b": "XqzeZGB7iU"}
if len(args) > 2:
    MODELS = {k: MODELS[k] for k in args[2:]}
NP = 12


def s(p, off=0.0):                     # the beat's pulse: 1 on the beat, 0 half way (2 beats a cycle)
    return 0.5 + 0.5 * math.cos(2 * math.pi * (p / 6.0 - off))


def w(p, off=0.0):                     # a 2-beat swing -1..1
    return math.sin(2 * math.pi * (p / 12.0 - off))


def arms(rx_r, rz_r, ex_r, ez_r, rx_l, rz_l, ex_l, ez_l):
    return {"UpperArm.R": (rx_r, 0, rz_r), "LowerArm.R": (ex_r, 0, ez_r),
            "UpperArm.L": (rx_l, 0, rz_l), "LowerArm.L": (ex_l, 0, ez_l)}


def move(name, p):
    """(base action or None, {bone: (x, y, z) degrees}, lift in metres)"""
    b = s(p)
    if name == "desk":                 # seated at the PC: nodding, drumming the desk
        d = arms(-62, 40 + 6 * s(p, 0.25), 0, 55 + 14 * s(p, 0.25), -62, 40 + 6 * s(p, 0.75), 0, 55 + 14 * s(p, 0.75))
        d.update({"Head": (12 * b - 4, 0, 4 * w(p)), "Neck": (6 * b, 0, 0), "Torso": (6 + 4 * b, 0, 3 * w(p))})
        return "Sitting", d, 0.0
    if name == "seat_cheer":           # seated, one fist pumping
        d = arms(70 - 40 * b, 10, 30 + 40 * b, 0, -62, 40, 0, 50)
        d.update({"Head": (10 * b - 8, 0, 0), "Torso": (4 * b, 0, -4)})
        return "Sitting", d, 0.0
    if name == "bob":                  # standing, bobbing, arms loose
        d = arms(-72, 8 + 6 * w(p), 0, 18 + 6 * w(p), -72, 8 - 6 * w(p), 0, 18 - 6 * w(p))
        d.update({"Head": (14 * b - 6, 0, 0), "Torso": (5 * b, 0, 4 * w(p))})
        return None, d, -0.04 * b
    if name == "pump":                 # the right fist pumped on the beat
        d = arms(30 + 45 * (1 - b), 25, 60 * b, 0, -70, 10, 0, 30)
        d.update({"Head": (10 * b - 10, 0, 0), "Torso": (-4 + 4 * b, 0, -3)})
        return None, d, -0.05 * b
    if name == "cheer":                # both arms up, pumping
        d = arms(72 - 22 * b, 12, 10 + 50 * b, 0, 72 - 22 * b, 12, 10 + 50 * b, 0)
        d.update({"Head": (-14 + 6 * b, 0, 0), "Torso": (-6, 0, 0)})
        return None, d, -0.05 * b
    if name == "jump":                 # arms up, jumping on the beat
        up = 1 - b
        d = arms(78, 6, 8, 0, 78, 6, 8, 0)
        d.update({"Head": (-10, 0, 0), "Torso": (-4, 0, 0)})
        return None, d, 0.22 * up * up - 0.05
    if name == "sway":                 # arms overhead, swaying side to side
        d = arms(70, 10, 20, 0, 70, 10, 20, 0)
        d.update({"Torso": (-4, 0, 12 * w(p)), "Head": (-8, 0, 8 * w(p))})
        return None, d, -0.03 * b
    if name == "point":                # pointing at the screen, bouncing
        d = arms(25 + 6 * b, 70, 0, 0, -70, 10, 0, 25)
        d.update({"Head": (-6 + 6 * b, 0, 0), "Torso": (2, 0, -4)})
        return None, d, -0.04 * b
    if name == "raise":                # one arm held up (a can, a lighter, a phone), the other loose, bobbing
        d = arms(62, 25, 50, 0, -72, 8, 0, 20)
        d.update({"Head": (12 * b - 6, 0, 0), "Torso": (4 * b, 0, -3)})
        return None, d, -0.04 * b
    if name == "dance":                # fists up by the shoulders, pumping alternately, the body twisting
        d = arms(-10 + 25 * s(p, 0.0), 20, 95, 10, -10 + 25 * s(p, 0.5), 20, 95, 10)
        d.update({"Torso": (4, 14 * w(p), 5 * w(p)), "Head": (10 * b - 4, -8 * w(p), 0)})
        return None, d, -0.05 * b
    raise KeyError(name)


MOVES = ["desk", "seat_cheer", "bob", "pump", "cheer", "jump", "sway", "point", "raise", "dance"]


def mirror(bone, e):
    x, y, z = e
    if bone.endswith(".L"):
        return (x, -y, -z)
    return e


def bake(arm, meshes):
    dg = bpy.context.evaluated_depsgraph_get()
    vs = []
    for o in meshes:
        me = o.evaluated_get(dg).to_mesh()
        vs.append(np.array([tuple(o.matrix_world @ v.co) for v in me.vertices]))
        o.evaluated_get(dg).to_mesh_clear()
    return np.concatenate(vs)


out = {}
for key, mid in MODELS.items():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=f"{DL}/{mid}.glb")
    arm = [o for o in bpy.data.objects if o.type == 'ARMATURE'][0]
    meshes = [o for o in bpy.data.objects if o.type == 'MESH' and any(m.type == 'ARMATURE' for m in o.modifiers)]
    tris, base = [], 0
    for o in meshes:
        o.data.calc_loop_triangles()
        tris.append(np.array([tuple(t.vertices) for t in o.data.loop_triangles]) + base)
        base += len(o.data.vertices)
    out[key + "/t"] = np.concatenate(tris)
    actions = {a.name.split("_")[-1]: a for a in bpy.data.actions}
    for mv in MOVES:
        frames, hands = [], []
        for p in range(NP):
            act, bones, lift = move(mv, p)
            arm.animation_data.action = actions[act] if act else None
            for pb in arm.pose.bones:       # back to the rest pose (the clip then sets the bones it keys)
                if True:
                    pb.rotation_mode = 'QUATERNION'
                    pb.rotation_quaternion = (1, 0, 0, 0)
                    pb.location = (0, 0, 0)
            bpy.context.scene.frame_set(1)       # force the clip to be evaluated afresh (no carried-over overrides)
            bpy.context.scene.frame_set(100 if act else 0)
            for bn, e in bones.items():
                pb = arm.pose.bones[bn]
                pb.rotation_mode = 'QUATERNION'
                q = Euler([math.radians(a) for a in mirror(bn, e)]).to_quaternion()
                if act and bn in ("Head", "Neck", "Torso"):
                    q = pb.rotation_quaternion @ q
                pb.rotation_quaternion = q
            bpy.context.view_layer.update()
            v = bake(arm, meshes)
            frames.append(v)
            hands.append([tuple(arm.matrix_world @ arm.pose.bones["Fingers." + sd].head) for sd in "RL"])
        v = np.stack(frames)
        out[f"{key}/{mv}"] = v
        out[f"{key}/{mv}/hands"] = np.array(hands)
        out[f"{key}/{mv}/lift"] = np.array([move(mv, p)[2] for p in range(NP)])
        print("POSED", key, mv, v.shape)
np.savez_compressed(OUT, **out)
