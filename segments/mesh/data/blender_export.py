"""Run inside Blender (headless): writes a model's triangles, at each subdivision level asked for, as .npz.

    blender -b [MODEL.blend] -P blender_export.py -- OUT_PREFIX SOURCE LEVELS...

SOURCE is `suzanne` (Blender's own monkey, Shift+A > Mesh > Monkey), `cube` (the default cube), a .glb/.gltf/.obj
file, or `scene` (the meshes of the .blend opened). LEVELS are Catmull-Clark subdivision levels (0 is the model as it
is). Each level writes OUT_PREFIX_L.npz: vertices (n, 3), triangles (m, 3), vertex normals (n, 3), corner UVs (m, 3, 2)
when the model has UVs, the polygon each triangle comes from (m), and the polygon count.
"""
import sys

import bpy
import numpy as np

argv = sys.argv[sys.argv.index("--") + 1:]
prefix, source, levels = argv[0], argv[1], [int(a) for a in argv[2:]]

if source in ("suzanne", "cube"):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    if source == "suzanne":
        bpy.ops.mesh.primitive_monkey_add(size=2.0, calc_uvs=True)
    else:
        bpy.ops.mesh.primitive_cube_add(size=2.0, calc_uvs=True)
    objs = [bpy.context.active_object]
elif source != "scene":
    bpy.ops.wm.read_factory_settings(use_empty=True)
    if source.lower().endswith((".glb", ".gltf")):
        bpy.ops.import_scene.gltf(filepath=source)
    elif source.lower().endswith(".obj"):
        bpy.ops.wm.obj_import(filepath=source)
    else:
        raise SystemExit("unknown source " + source)
    objs = [o for o in bpy.context.scene.objects if o.type == "MESH"]
else:
    objs = [o for o in bpy.context.scene.objects if o.type == "MESH"]

for level in levels:
    vs, ts, ns, uvs, polys, npoly = [], [], [], [], [], 0
    for obj in objs:
        mod = None
        if level > 0:
            mod = obj.modifiers.new("subdiv", "SUBSURF")
            mod.levels = level
            mod.render_levels = level
        dg = bpy.context.evaluated_depsgraph_get()
        ev = obj.evaluated_get(dg)
        me = ev.to_mesh()
        me.transform(obj.matrix_world)
        me.calc_loop_triangles()
        nv = len(me.vertices)
        v = np.zeros(nv * 3, np.float64)
        me.vertices.foreach_get("co", v)
        n = np.zeros(nv * 3, np.float64)
        me.vertices.foreach_get("normal", n)
        nt = len(me.loop_triangles)
        tv = np.zeros(nt * 3, np.int64)
        me.loop_triangles.foreach_get("vertices", tv)
        tl = np.zeros(nt * 3, np.int64)
        me.loop_triangles.foreach_get("loops", tl)
        tp = np.zeros(nt, np.int64)
        me.loop_triangles.foreach_get("polygon_index", tp)
        base = sum(len(a) for a in vs)
        vs.append(v.reshape(-1, 3)); ns.append(n.reshape(-1, 3))
        ts.append(tv.reshape(-1, 3) + base); polys.append(tp + npoly)
        if me.uv_layers.active is not None:
            uv = np.zeros(len(me.loops) * 2, np.float64)
            me.uv_layers.active.data.foreach_get("uv", uv)
            uvs.append(uv.reshape(-1, 2)[tl.reshape(-1, 3)])
        npoly += len(me.polygons)
        ev.to_mesh_clear()
        if mod is not None:
            obj.modifiers.remove(mod)
    out = dict(vertices=np.concatenate(vs), triangles=np.concatenate(ts), normals=np.concatenate(ns),
               polygon=np.concatenate(polys), polygons=np.array(npoly))
    if len(uvs) == len(objs):
        out["uvs"] = np.concatenate(uvs)
    np.savez(f"{prefix}_{level}.npz", **out)
    print(f"level {level}: {len(out['vertices'])} vertices, {len(out['triangles'])} triangles, {npoly} polygons")
