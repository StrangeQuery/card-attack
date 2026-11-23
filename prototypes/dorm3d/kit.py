"""What room.py (the dorm) and hall.py (the screening hall) share: an empty scene, the camera matched to a prototype's
projection, matte materials, simple geometry in room metres, MakeHuman people (MPFB 2, CC0 assets and poses), Cycles
and the multilayer renders. Room point (X across, Y up, d ahead of the eye) is Blender (X, d, Y).
"""
import math
import os

import bpy
import bmesh
from mathutils import Matrix, Vector, noise

W, H = 1536, 768
F = HZ = EY = None
sc = None


def init(f, hz, ey):
    """An empty scene; the camera's focal length (pixels on a 1536-wide frame), horizon row (of 768), eye height."""
    global F, HZ, EY, sc
    F, HZ, EY = f, hz, ey
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    return sc


def V(X, Y, d):
    return Vector((X, d, Y))


def P(X, Y, d):
    return (W / 2 + F * X / d, HZ - F * (Y - EY) / d)


# ---- materials: all matte (no specular, no sheen, no coat) -------------------------------------------------------
def _bsdf(m):
    m.use_nodes = True
    nt = m.node_tree
    b = nt.nodes["Principled BSDF"]
    b.inputs["Roughness"].default_value = 1.0
    b.inputs["Specular IOR Level"].default_value = 0.0
    return nt, b


def mat(name, col, emit=None, strength=0.0):
    m = bpy.data.materials.new(name)
    nt, b = _bsdf(m)
    b.inputs["Base Color"].default_value = (*col, 1)
    if emit:
        b.inputs["Emission Color"].default_value = (*emit, 1)
        b.inputs["Emission Strength"].default_value = strength
    return m


def mat_noise(name, c0, c1, scale=20.0, detail=4.0, bump=0.0, coords="Object"):
    """Two colours mixed by noise (paint, fabric, cardboard, tile grain)."""
    m = bpy.data.materials.new(name)
    nt, b = _bsdf(m)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    nz = nt.nodes.new("ShaderNodeTexNoise"); nz.inputs["Scale"].default_value = scale; nz.inputs["Detail"].default_value = detail
    nt.links.new(tc.outputs[coords], nz.inputs["Vector"])
    ramp = nt.nodes.new("ShaderNodeValToRGB")
    ramp.color_ramp.elements[0].color = (*c0, 1); ramp.color_ramp.elements[1].color = (*c1, 1)
    ramp.color_ramp.elements[0].position = 0.35; ramp.color_ramp.elements[1].position = 0.65
    nt.links.new(nz.outputs["Fac"], ramp.inputs["Fac"])
    nt.links.new(ramp.outputs["Color"], b.inputs["Base Color"])
    if bump:
        bp = nt.nodes.new("ShaderNodeBump"); bp.inputs["Strength"].default_value = bump
        nt.links.new(nz.outputs["Fac"], bp.inputs["Height"]); nt.links.new(bp.outputs["Normal"], b.inputs["Normal"])
    return m


def mat_wood(name, c0, c1, scale=6.0):
    """Wood: grain along the piece's long axis (one material per axis; box() picks)."""
    out = {}
    for ax in range(3):
        m = bpy.data.materials.new(f"{name}_{'XYZ'[ax]}")
        nt, b = _bsdf(m)
        tc = nt.nodes.new("ShaderNodeTexCoord")
        mp = nt.nodes.new("ShaderNodeMapping")
        sc_ = [scale * 6] * 3; sc_[ax] = scale * 0.25
        mp.inputs["Scale"].default_value = sc_
        nt.links.new(tc.outputs["Object"], mp.inputs["Vector"])
        nz = nt.nodes.new("ShaderNodeTexNoise"); nz.inputs["Scale"].default_value = 1.0; nz.inputs["Detail"].default_value = 6.0
        nz.inputs["Distortion"].default_value = 0.4
        nt.links.new(mp.outputs["Vector"], nz.inputs["Vector"])
        ramp = nt.nodes.new("ShaderNodeValToRGB")
        ramp.color_ramp.elements[0].color = (*c0, 1); ramp.color_ramp.elements[1].color = (*c1, 1)
        ramp.color_ramp.elements[0].position = 0.3; ramp.color_ramp.elements[1].position = 0.7
        nt.links.new(nz.outputs["Fac"], ramp.inputs["Fac"]); nt.links.new(ramp.outputs["Color"], b.inputs["Base Color"])
        out[ax] = m
    return out


def mat_brick(name, brick, mortar, bw, bh, coords="Object", mapping=None, bump=0.3):
    """Cinder block / tiles: a brick texture in metres."""
    m = bpy.data.materials.new(name)
    nt, b = _bsdf(m)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    br = nt.nodes.new("ShaderNodeTexBrick")
    br.inputs["Scale"].default_value = 1.0
    br.inputs["Brick Width"].default_value = bw; br.inputs["Row Height"].default_value = bh
    br.inputs["Mortar Size"].default_value = 0.008
    br.inputs["Color1"].default_value = (*brick, 1); br.inputs["Color2"].default_value = (*[c * 0.96 for c in brick], 1)
    br.inputs["Mortar"].default_value = (*mortar, 1)
    src = tc.outputs[coords]
    if mapping:
        mp = nt.nodes.new("ShaderNodeMapping"); mp.inputs["Rotation"].default_value = mapping
        nt.links.new(src, mp.inputs["Vector"]); src = mp.outputs["Vector"]
    nt.links.new(src, br.inputs["Vector"])
    nz = nt.nodes.new("ShaderNodeTexNoise"); nz.inputs["Scale"].default_value = 60.0
    mix = nt.nodes.new("ShaderNodeMix"); mix.data_type = "RGBA"; mix.blend_type = "MULTIPLY"
    mix.inputs["Factor"].default_value = 0.12
    nt.links.new(br.outputs["Color"], mix.inputs["A"]); nt.links.new(nz.outputs["Color"], mix.inputs["B"])
    nt.links.new(mix.outputs["Result"], b.inputs["Base Color"])
    bp = nt.nodes.new("ShaderNodeBump"); bp.inputs["Strength"].default_value = bump; bp.invert = True
    nt.links.new(br.outputs["Fac"], bp.inputs["Height"]); nt.links.new(bp.outputs["Normal"], b.inputs["Normal"])
    return m


def mat_image(name, path, alpha=False):
    m = bpy.data.materials.new(name)
    nt, b = _bsdf(m)
    tx = nt.nodes.new("ShaderNodeTexImage"); tx.image = bpy.data.images.load(path)
    nt.links.new(tx.outputs["Color"], b.inputs["Base Color"])
    if alpha:
        nt.links.new(tx.outputs["Alpha"], b.inputs["Alpha"])
    return m


# ---- geometry helpers ----------------------------------------------------------------------------------------------
def link(ob):
    sc.collection.objects.link(ob)
    return ob


def mesh_ob(name, vs, fs, m):
    me = bpy.data.meshes.new(name)
    me.from_pydata(vs, [], fs)
    me.update()
    ob = link(bpy.data.objects.new(name, me))
    if m:
        me.materials.append(m)
    return ob


def box(name, x0, x1, y0, y1, d0, d1, m, bevel=0.0, seg=2):
    if isinstance(m, dict):            # wood: the grain along the longest side (Blender axes: X, d, up)
        m = m[max(range(3), key=lambda i: (abs(x1 - x0), abs(d1 - d0), abs(y1 - y0))[i])]
    vs = [V(x, y, d) for x in (x0, x1) for y in (y0, y1) for d in (d0, d1)]
    fs = [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)]
    ob = mesh_ob(name, vs, fs, m)
    _uv_cube(ob)
    if bevel:
        bv = ob.modifiers.new("bevel", "BEVEL"); bv.width = bevel; bv.segments = seg
    return ob


def _uv_cube(ob):
    bm = bmesh.new(); bm.from_mesh(ob.data)
    uv = bm.loops.layers.uv.new("UVMap")
    for f in bm.faces:
        n = f.normal
        ax = max(range(3), key=lambda i: abs(n[i]))
        a, b_ = [(1, 2), (0, 2), (0, 1)][ax]
        for l in f.loops:
            l[uv].uv = (l.vert.co[a], l.vert.co[b_])
    bm.to_mesh(ob.data); bm.free()


def quad(name, pts, m, uv=((0, 0), (1, 0), (1, 1), (0, 1))):
    """A picture quad: pts top-left, top-right, bottom-right, bottom-left (room metres); the image upright."""
    ob = mesh_ob(name, [V(*p) for p in pts], [(3, 2, 1, 0)], m)
    me = ob.data
    lay = me.uv_layers.new(name="UVMap")
    for li, vi in zip(me.polygons[0].loop_indices, (3, 2, 1, 0)):
        lay.data[li].uv = ((0, 1), (1, 1), (1, 0), (0, 0))[vi]
    return ob


def cyl(name, X, y0, y1, d, r, m, n=24, r1=None, cap=True):
    r1 = r if r1 is None else r1
    vs = [V(X + rr * math.cos(2 * math.pi * i / n), y, d + rr * math.sin(2 * math.pi * i / n)) for (y, rr) in ((y0, r), (y1, r1)) for i in range(n)]
    fs = [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    if cap:
        fs += [tuple(range(n))[::-1], tuple(range(n, 2 * n))]
    ob = mesh_ob(name, vs, fs, m)
    for p in ob.data.polygons:
        p.use_smooth = True
    return ob


def blob(name, X, Y, d, sx, sy, sd, m, seed=0, rough=0.35, subdiv=3):
    """A lumpy thing (a crumpled napkin, a pillow, a wad of clothes): a displaced icosphere."""
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=1.0)
    off = Vector((seed * 7.1, seed * 3.3, seed * 5.7))
    for v in bm.verts:
        n = noise.noise(v.co * 1.7 + off) * rough + noise.noise(v.co * 4.3 + off) * rough * 0.4
        v.co = v.co * (1 + n)
        v.co = Vector((v.co.x * sx, v.co.z * sd, v.co.y * sy))
        v.co += V(X, Y, d)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    ob = link(bpy.data.objects.new(name, me))
    me.materials.append(m)
    for p in me.polygons:
        p.use_smooth = True
    return ob


def tube(name, pts, r, m, taper=None, res=12):
    """A soft tube along room points (a sock, a sleeve, a cable)."""
    cu = bpy.data.curves.new(name, "CURVE"); cu.dimensions = "3D"
    cu.bevel_depth = r; cu.bevel_resolution = 4; cu.resolution_u = res; cu.use_fill_caps = True
    sp = cu.splines.new("NURBS"); sp.points.add(len(pts) - 1)
    for i, p in enumerate(pts):
        sp.points[i].co = (*V(*p), 1)
        if taper:
            sp.points[i].radius = taper[i]
    sp.use_endpoint_u = True; sp.order_u = 3
    ob = link(bpy.data.objects.new(name, cu))
    cu.materials.append(m)
    return ob


def rot_about(ob, axis_pt, angle, axis="Z"):
    ob.matrix_world = Matrix.Translation(axis_pt) @ Matrix.Rotation(angle, 4, axis) @ Matrix.Translation(-axis_pt) @ ob.matrix_world




# ---- people (MakeHuman: MPFB 2 and its CC0 system assets, CC0 poses) ------------------------------------------------
def matte(ob):
    for slot in ob.material_slots:
        m = slot.material
        if not m or not m.use_nodes:
            continue
        for n in m.node_tree.nodes:
            if n.type == "BSDF_PRINCIPLED":
                for k_, v in (("Specular IOR Level", 0.0), ("Roughness", 1.0), ("Coat Weight", 0.0),
                              ("Sheen Weight", 0.0), ("Subsurface Weight", 0.0)):
                    if k_ in n.inputs:
                        for l in list(n.inputs[k_].links):
                            m.node_tree.links.remove(l)
                        n.inputs[k_].default_value = v




def human(p, X, z, d, yaw):
    """A MakeHuman character (p: gender, weight, race, skin, hair, clothes, pose, optional height), posed, facing +d
    turned by yaw, its hips at (X, z, d); its new objects."""
    from bl_ext.user_default.mpfb.services.animationservice import AnimationService
    from bl_ext.user_default.mpfb.services.humanservice import HumanService
    from bl_ext.user_default.mpfb.services.locationservice import LocationService
    data = LocationService.get_user_data()
    macro = {"gender": p["gender"], "age": p.get("age", 0.42), "muscle": 0.5, "weight": p["weight"], "proportions": 0.5,
             "height": p.get("height", 0.5), "cupsize": 0.5, "firmness": 0.5,
             "race": dict(zip(("asian", "caucasian", "african"), p["race"]))}
    before = set(bpy.data.objects)
    b = HumanService.create_human(macro_detail_dict=macro)
    rig = HumanService.add_builtin_rig(b, "default")
    HumanService.set_character_skin(os.path.join(data, "skins", p["skin"], p["skin"] + ".mhmat"), b, skin_type="GAMEENGINE")
    HumanService.add_mhclo_asset(os.path.join(data, "hair", p["hair"], p["hair"] + ".mhclo"), b, asset_type="Hair", material_type="MAKESKIN")
    for c in p["clothes"]:
        HumanService.add_mhclo_asset(os.path.join(data, "clothes", c, c + ".mhclo"), b, asset_type="Clothes", material_type="MAKESKIN")
    AnimationService.import_bvh_file_as_pose(rig, os.path.join(data, "poses", p["pose"], p["pose"] + ".bvh"))
    bpy.context.view_layer.update()
    rig.matrix_world = Matrix.Rotation(math.pi - yaw, 4, "Z") @ rig.matrix_world
    bpy.context.view_layer.update()
    hip = rig.matrix_world @ rig.pose.bones["root"].head
    rig.location += V(X, z, d) - hip
    bpy.context.view_layer.update()
    new = set(bpy.data.objects) - before
    for ob in new:
        if ob.type == "MESH":
            matte(ob)
    return new


def frozen(objs, name):
    """A posed character's meshes baked into one static mesh (for instancing round a crowd); the originals removed."""
    dg = bpy.context.evaluated_depsgraph_get()
    bm = bmesh.new()
    mats = []
    for ob in objs:
        if ob.type != "MESH":
            continue
        me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
        me.transform(ob.matrix_world)
        base = len(mats)
        mats += [s.material for s in ob.material_slots] or [None]
        for poly in me.polygons:
            poly.material_index += base
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me); bm.free()
    for m in mats:
        me.materials.append(m)
    for ob in objs:
        bpy.data.objects.remove(ob, do_unlink=True)
    return me


# ---- camera, Cycles, the renders --------------------------------------------------------------------------------------
def camera():
    cam = link(bpy.data.objects.new("cam", bpy.data.cameras.new("cam")))
    sc.camera = cam
    cam.location = V(0, EY, 0); cam.rotation_euler = (math.radians(90), 0, 0)
    cam.data.sensor_fit = "HORIZONTAL"; cam.data.sensor_width = 36.0
    cam.data.lens = F / W * 36.0
    cam.data.shift_y = -(H / 2 - HZ) / W
    cam.data.clip_start = 0.05
    return cam


def cycles(samples):
    w = bpy.data.worlds.new("w"); sc.world = w; w.color = (0, 0, 0)
    sc.render.engine = "CYCLES"
    prefs = bpy.context.preferences.addons["cycles"].preferences
    try:
        prefs.compute_device_type = "METAL"; prefs.get_devices()
        for dev in prefs.devices:
            dev.use = True
        sc.cycles.device = "GPU"
    except Exception as e:
        print("no GPU", e)
    sc.cycles.samples = samples
    sc.cycles.use_denoising = True
    sc.cycles.max_bounces = 6
    sc.render.film_transparent = False
    vl = sc.view_layers[0]
    vl.use_pass_diffuse_color = True
    vl.use_pass_object_index = True
    sc.render.image_settings.media_type = "MULTI_LAYER_IMAGE"
    sc.render.image_settings.file_format = "OPEN_EXR_MULTILAYER"
    sc.render.image_settings.color_depth = "32"
    return w


def render(path, scale, box=None):
    sc.render.resolution_x, sc.render.resolution_y = W * scale, H * scale
    sc.render.resolution_percentage = 100
    if box:
        x0, y0, x1, y1 = box
        sc.render.use_border = True; sc.render.use_crop_to_border = True
        sc.render.border_min_x, sc.render.border_max_x = x0 / W, x1 / W
        sc.render.border_min_y, sc.render.border_max_y = 1 - y1 / H, 1 - y0 / H
    else:
        sc.render.use_border = False
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)
