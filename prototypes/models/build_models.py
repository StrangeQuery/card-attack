"""Builds the bullet train's models in Blender and exports them as triangle meshes (.obj), for train.py now and the
mesh kit (segments/mesh) later. Everything is made here from numbers: no downloaded models (Poly Haven's trees run to
millions of triangles; nothing CC0 fit the Japanese suburb or the N700 better than building them).

    blender -b --factory-startup -P build_models.py -- OUTDIR

Coordinates are the film's: metres, y up, the line along +z. Each model is centred on its footprint at y = 0 (the
ground it stands on). Material names are roles that train.py maps to its materials:
  body, screen, lamp, stripe, skirt, winband, gangway (the train); wall, roof, tin, trim, rail, glass, unit (buildings);
  leaf, trunk (trees); steel, conc (masts, poles, the bridge).
"""
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector, noise

OUT = sys.argv[sys.argv.index("--") + 1] if "--" in sys.argv else "models"
os.makedirs(OUT, exist_ok=True)
COUNTS = {}

def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)

class Mesh:
    """A bmesh with faces tagged by role."""
    def __init__(self):
        self.bm = bmesh.new()
        self.roles = []

    def role(self, r):
        if r not in self.roles:
            self.roles.append(r)
        return self.roles.index(r)

    def face(self, pts, r):
        vs = [self.bm.verts.new(p) for p in pts]
        f = self.bm.faces.new(vs)
        f.material_index = self.role(r)
        return f

    def box(self, lo, hi, r, bottom=False):
        (x0, y0, z0), (x1, y1, z1) = lo, hi
        F = [((x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)), ((x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1)),
             ((x0, y0, z0), (x0, y1, z0), (x1, y1, z0), (x1, y0, z0)), ((x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)),
             ((x0, y1, z0), (x0, y1, z1), (x1, y1, z1), (x1, y1, z0))]
        if bottom:
            F.append(((x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)))
        for f in F:
            self.face(f, r)

    def member(self, a, b, w, h, r, sides=4):
        """A beam from a to b, w by h (sides=4) or round (sides > 4, w the diameter)."""
        a, b = Vector(a), Vector(b)
        t = (b - a).normalized()
        s = t.cross(Vector((0, 1, 0)))
        if s.length < 1e-3:
            s = Vector((1, 0, 0))
        s.normalize(); u = s.cross(t)
        if sides == 4:
            cs = [(-1, -1), (1, -1), (1, 1), (-1, 1)]
            ring = lambda p: [p + s * (w / 2) * i + u * (h / 2) * j for i, j in cs]
        else:
            ring = lambda p: [p + s * (w / 2) * math.cos(2 * math.pi * k / sides) + u * (w / 2) * math.sin(2 * math.pi * k / sides)
                              for k in range(sides)]
        A, B = ring(a), ring(b)
        n = len(A)
        for k in range(n):
            self.face((A[k], A[(k + 1) % n], B[(k + 1) % n], B[k]), r)
        self.face(list(reversed(A)), r)
        self.face(B, r)

    def obj(self, name, smooth=False):
        bmesh.ops.remove_doubles(self.bm, verts=self.bm.verts, dist=1e-5)
        bmesh.ops.recalc_face_normals(self.bm, faces=self.bm.faces)
        me = bpy.data.meshes.new(name)
        self.bm.to_mesh(me)
        self.bm.free()
        for r in self.roles:
            m = bpy.data.materials.get(r) or bpy.data.materials.new(r)
            me.materials.append(m)
        for p in me.polygons:
            p.use_smooth = smooth
        ob = bpy.data.objects.new(name, me)
        bpy.context.collection.objects.link(ob)
        return ob

def export(name, objs):
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    if len(objs) > 1:
        bpy.ops.object.join()
    ob = bpy.context.view_layer.objects.active
    bpy.ops.object.convert(target="MESH")                 # apply the modifiers
    me = ob.data
    bm = bmesh.new(); bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces)
    COUNTS[name] = len(bm.faces)
    bm.to_mesh(me); bm.free()
    path = os.path.join(OUT, name + ".obj")
    bpy.ops.wm.obj_export(filepath=path, export_selected_objects=True, forward_axis="Y", up_axis="Z",
                          export_materials=True, export_triangulated_mesh=True, export_normals=True, export_uv=False,
                          apply_modifiers=True)
    mtl = path[:-4] + ".mtl"
    if os.path.exists(mtl):
        os.remove(mtl)

def smoothstep(a, b, x):
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)

# ---- the N700-style train ---------------------------------------------------------------------------------------

NOSE, CAR = 10.7, 25.0
HW = 1.69                                                 # half the body's width

def nose_profile(d):
    """Half-width, the shoulders' ("wings'") height and the cab canopy's at d metres back from the tip."""
    s = min(1.0, max(0.0, d / NOSE))
    hw = HW * (0.60 + 0.40 * (1 - (1 - s) ** 2.4))
    wing = 0.80 + 2.80 * s ** 1.6                          # a long, low, slightly concave bill
    canopy = wing + 0.95 * smoothstep(0.30, 0.64, s) * (1 - smoothstep(0.82, 1.0, s))
    return hw, wing, canopy

# the cross-section's corners, around from the bottom middle: (across as a share of the half-width, up as a share
# of the height), the body's; the nose squashes them to its profile
SECTION = [(0.0, 0.0), (0.55, 0.0), (0.80, 0.02), (0.93, 0.07), (0.975, 0.13), (0.99, 0.208), (1.0, 0.304),
           (1.0, 0.336), (1.0, 0.357), (1.0, 0.45), (0.996, 0.557), (0.985, 0.714), (0.94, 0.82), (0.86, 0.91),
           (0.72, 0.97), (0.50, 0.995), (0.25, 1.0), (0.0, 1.0)]
BOT = 0.30

def ring(d, top_body=3.62):
    """Points round the section at d metres from the nose's tip (d >= NOSE: the body)."""
    hw, wing, canopy = nose_profile(d)
    pts = []
    full = SECTION + [(-x, y) for x, y in reversed(SECTION[1:-1])]
    for ax, ay in full:
        x = hw * ax
        if d >= NOSE:
            top = top_body
        else:
            u = abs(ax) / 0.62
            top = wing + (canopy - wing) * math.exp(-u ** 6 * 1.2)
        y = BOT + (top - BOT) * ay
        pts.append((x, y))
    if d < 0.6:                                           # round off the very tip
        k = 0.55 + 0.45 * math.sqrt(d / 0.6)
        cy = BOT + (wing - BOT) * 0.45
        pts = [(x * k, cy + (y - cy) * k) for x, y in pts]
    return pts

def role_at(x, y, d, hw, wing, canopy, body):
    """Which material a face of the train is, from where its middle is."""
    if not body:
        u = abs(x) / 0.46 / max(hw, 1e-6)
        top = wing + (canopy - wing) * math.exp(-u ** 4 * 1.2)
        if NOSE * 0.50 < d < NOSE * 0.70 and y > top - 0.42 and abs(x) < hw * 0.40:
            return "screen"
        if 1.2 < d < 3.0 and abs(abs(x) - 0.72 * hw) < 0.26 and abs(y - 0.70) < 0.13:
            return "lamp"
    if not body:
        return "nose"                                     # the stripe down the nose is drawn per pixel
    sy = 1.15
    if abs(y - sy) < 0.16 or abs(y - (sy + 0.30)) < 0.036:
        return "stripe"
    if body and 2.15 < y < 2.67:
        return "winband"
    if y < 0.62:
        return "skirt"
    return "body"

def train_car(name, nose):
    """A car 25 m long, its front at z = 0 (with the nose if nose), its back at z = CAR."""
    M = Mesh()
    zs = []
    if nose:
        zs += [0.0, 0.1, 0.3, 0.6] + [1.0 + 0.55 * k for k in range(int((NOSE - 1.0) / 0.55) + 1)] + [NOSE]
        zs += [NOSE + 1.0, 12.0, 18.0, CAR - 0.25]
    else:
        zs = [0.25, 6.0, 12.5, 19.0, CAR - 0.25]
    zs = sorted(set(round(z, 3) for z in zs))
    rings = []
    for z in zs:
        d = z if nose else 99.0
        rings.append([(x, y, z) for x, y in ring(d)])
    n = len(rings[0])
    for i in range(len(rings) - 1):
        for k in range(n):
            a, b = rings[i][k], rings[i][(k + 1) % n]
            c, e = rings[i + 1][(k + 1) % n], rings[i + 1][k]
            mx = (a[0] + b[0] + c[0] + e[0]) / 4; my = (a[1] + b[1] + c[1] + e[1]) / 4; mz = (a[2] + e[2]) / 2
            d = mz if nose else 99.0
            hw, wing, canopy = nose_profile(d)
            r = role_at(mx, my, d, hw, wing, canopy, d >= NOSE)
            M.face((a, e, c, b), r)
    # close the ends: the tip (nose) and the gangway end
    if nose:
        cy = sum(p[1] for p in rings[0]) / n
        tip = (0.0, cy, -0.02)
        for k in range(n):
            M.face((rings[0][(k + 1) % n], rings[0][k], tip), "body")
    else:
        c0 = (0.0, 1.9, zs[0])
        for k in range(n):
            M.face((rings[0][(k + 1) % n], rings[0][k], c0), "body")
    c1 = (0.0, 1.9, zs[-1])
    for k in range(n):
        M.face((rings[-1][k], rings[-1][(k + 1) % n], c1), "body")
    M.box((-1.55, 0.45, CAR - 0.25), (1.55, 3.45, CAR + 0.25), "gangway")
    for zb in (3.0, CAR - 3.0):                           # the bogies, dark under the skirt
        M.box((-1.3, 0.05, zb - 1.6), (1.3, 0.45, zb + 1.6), "gangway")
    ob = M.obj(name, smooth=True)
    if nose:                                              # smooth the nose once over, keeping its edges' materials
        mod = ob.modifiers.new("sub", "SUBSURF")
        mod.levels = 1; mod.render_levels = 1
    return ob

def pantograph():
    M = Mesh()
    M.box((-0.9, 3.62, -0.6), (0.9, 3.85, 0.6), "gangway")
    M.member((0.0, 3.8, -0.4), (0.0, 4.6, 0.3), 0.08, 0.08, "gangway")
    M.member((0.0, 4.6, 0.3), (0.0, 4.95, 0.0), 0.08, 0.08, "gangway")
    M.box((-0.8, 4.92, -0.12), (0.8, 5.0, 0.12), "gangway")
    return M.obj("pantograph")

# ---- trees: displaced crowns, so the outline is a tree's, not a ball's ----------------------------------------

def crown(M_obj_name, centre, radii, seed, strength, sub=3, scale=1.2):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub, radius=1.0, location=(0, 0, 0))
    ob = bpy.context.active_object
    ob.name = M_obj_name
    me = ob.data
    for v in me.vertices:
        p = v.co.copy()
        n1 = noise.noise(p * scale + Vector((seed, seed * 1.7, seed * 0.3)))
        n2 = noise.noise(p * scale * 3.1 + Vector((seed * 2.1, 0.5, seed)))
        k = 1 + strength * (0.75 * n1 + 0.35 * n2)
        v.co = Vector((p.x * radii[0] * k + centre[0], p.y * radii[1] * k + centre[1], p.z * radii[2] * k + centre[2]))
    mat = bpy.data.materials.get("leaf") or bpy.data.materials.new("leaf")
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    return ob

def tree_broadleaf(name, seed, h=9.0, r=3.6):
    random.seed(seed)
    M = Mesh()
    M.member((0, 0, 0), (0, h * 0.5, 0), 0.38, 0.38, "trunk", sides=6)
    objs = []
    for k in range(3):
        a = random.uniform(0, 6.28)
        M.member((0, h * 0.38, 0), (math.cos(a) * r * 0.5, h * 0.66, math.sin(a) * r * 0.5), 0.2, 0.2, "trunk", sides=5)
    objs.append(M.obj(name + "_wood"))
    cy = h * 0.66
    objs.append(crown(name + "_c0", (0, cy, 0), (r * 0.8, h * 0.28, r * 0.8), seed, 0.35, 2))
    for k in range(random.randint(2, 4)):
        a = random.uniform(0, 6.28); el = random.uniform(-0.1, 0.8); d = r * random.uniform(0.45, 0.7)
        c = (math.cos(a) * d * math.cos(el), cy + d * 0.7 * math.sin(el), math.sin(a) * d * math.cos(el))
        s = r * random.uniform(0.38, 0.55)
        objs.append(crown(name + "_c%d" % (k + 1), c, (s, s * 0.8, s), seed + k * 3.3, 0.4, 1))
    export(name, objs)

def spire(name, base_y, h, R, seed, sub):
    """A tapering crown: a sphere drawn out into a cone, its outline jagged with branch tiers."""
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub, radius=1.0, location=(0, 0, 0))
    ob = bpy.context.active_object
    ob.name = name
    for v in ob.data.vertices:
        p = v.co.copy()
        t = (p.y + 1) / 2                                 # 0 at the bottom, 1 at the tip
        taper = (1 - t) ** 0.9 * (0.55 + 0.45 * math.sin(min(t * 4.0, 1.0) * math.pi / 2))
        tier = 1 + 0.22 * math.sin(t * 2 * math.pi * 6 + seed)
        n1 = noise.noise(p * 2.2 + Vector((seed, 0.3 * seed, 1.1)))
        k = taper * tier * (1 + 0.3 * n1)
        v.co = Vector((p.x * R * k, base_y + t * h, p.z * R * k))
    mat = bpy.data.materials.get("leaf") or bpy.data.materials.new("leaf")
    ob.data.materials.append(mat)
    for poly in ob.data.polygons:
        poly.use_smooth = True
    return ob

def tree_cedar(name, seed, h=15.0):
    """A sugi: a tall narrow spire."""
    random.seed(seed)
    M = Mesh()
    M.member((0, 0, 0), (0, h * 0.35, 0), 0.4, 0.4, "trunk", sides=6)
    R = h * random.uniform(0.15, 0.19)
    export(name, [M.obj(name + "_wood"), spire(name + "_s", h * 0.18, h * 0.82, R, seed, 3)])

def tree_pine(name, seed, h=8.0):
    random.seed(seed)
    M = Mesh()
    lean = (random.uniform(-1.5, 1.5), random.uniform(-1.5, 1.5))
    M.member((0, 0, 0), (lean[0] * 0.5, h * 0.5, lean[1] * 0.5), 0.34, 0.34, "trunk", sides=6)
    M.member((lean[0] * 0.5, h * 0.5, lean[1] * 0.5), (lean[0], h * 0.85, lean[1]), 0.28, 0.28, "trunk", sides=6)
    objs = [M.obj(name + "_wood")]
    for k in range(random.randint(3, 5)):
        t = random.uniform(0.45, 1.0)
        c = (lean[0] * t + random.uniform(-1.6, 1.6), h * (0.42 + 0.55 * t), lean[1] * t + random.uniform(-1.6, 1.6))
        w = h * random.uniform(0.2, 0.3)
        objs.append(crown(name + "_p%d" % k, c, (w, w * 0.33, w), seed + k * 2.7, 0.45, 2, 1.5))
    export(name, objs)

def tree_far(name, seed, sp):
    """The far version of a tree (beyond 300 m): one displaced crown and a stick."""
    M = Mesh()
    if sp == "cedar":
        h = 16.0
        M.member((0, 0, 0), (0, h * 0.3, 0), 0.4, 0.4, "trunk")
        objs = [M.obj(name + "_wood"), spire(name + "_c", h * 0.18, h * 0.82, h * 0.16, seed, 1)]
    elif sp == "pine":
        h = 9.0
        M.member((0, 0, 0), (0, h * 0.6, 0), 0.3, 0.3, "trunk")
        objs = [M.obj(name + "_wood"), crown(name + "_c", (0, h * 0.75, 0), (h * 0.3, h * 0.16, h * 0.3), seed, 0.4, 1, 1.5)]
    elif sp == "bamboo":
        h = 12.0
        objs = [crown(name + "_c", (0, h * 0.5, 0), (3.2, h * 0.5, 3.2), seed, 0.3, 1, 1.5)]
    else:
        h = 9.5
        M.member((0, 0, 0), (0, h * 0.5, 0), 0.38, 0.38, "trunk")
        objs = [M.obj(name + "_wood"), crown(name + "_c", (0, h * 0.66, 0), (3.6, h * 0.33, 3.6), seed, 0.35, 1, 1.2)]
    export(name, objs)

def tree_bamboo(name, seed, h=11.0):
    random.seed(seed)
    objs = []
    for k in range(6):
        ox, oz = random.uniform(-2.5, 2.5), random.uniform(-2.5, 2.5)
        hh = h * random.uniform(0.75, 1.1)
        objs.append(crown(name + "_b%d" % k, (ox, hh * 0.62, oz), (1.6, hh * 0.4, 1.6), seed + k * 1.3, 0.35, 2, 2.0))
    export(name, objs)

# ---- the suburb: houses, apartments, blocks ----------------------------------------------------------------------

def roof(M, x0, x1, z0, z1, y0, kind, pitch, ov, r):
    a0, a1, b0, b1 = x0 - ov, x1 + ov, z0 - ov, z1 + ov
    alongz = (z1 - z0) >= (x1 - x0)
    half = ((x1 - x0) if alongz else (z1 - z0)) / 2 + ov
    y1 = y0 + pitch * half
    th = 0.12                                             # the roof's thickness shows at the eaves
    if alongz:
        xc = (a0 + a1) / 2
        r0, r1 = (b0 + half, b1 - half) if kind == "hip" else (b0, b1)
        P1, P2 = (xc, y1, r0), (xc, y1, r1)
        M.face(((a0, y0, b0), (a0, y0, b1), P2, P1), r)
        M.face(((a1, y0, b0), P1, P2, (a1, y0, b1)), r)
        M.face(((a0, y0 - th, b0), (a0, y0 - th, b1), (a0, y0, b1), (a0, y0, b0)), "trim")
        M.face(((a1, y0 - th, b0), (a1, y0, b0), (a1, y0, b1), (a1, y0 - th, b1)), "trim")
        if kind == "hip":
            M.face(((a0, y0, b0), P1, (a1, y0, b0)), r)
            M.face(((a0, y0, b1), (a1, y0, b1), P2), r)
        else:
            for ze in (z0, z1):
                M.face(((x0, y0, ze), (xc, y1 - 0.1, ze), (x1, y0, ze)), "wall")
    else:
        zc = (b0 + b1) / 2
        r0, r1 = (a0 + half, a1 - half) if kind == "hip" else (a0, a1)
        P1, P2 = (r0, y1, zc), (r1, y1, zc)
        M.face(((a0, y0, b0), (a1, y0, b0), P2, P1), r)
        M.face(((a0, y0, b1), P1, P2, (a1, y0, b1)), r)
        M.face(((a0, y0 - th, b0), (a1, y0 - th, b0), (a1, y0, b0), (a0, y0, b0)), "trim")
        M.face(((a0, y0 - th, b1), (a0, y0, b1), (a1, y0, b1), (a1, y0 - th, b1)), "trim")
        if kind == "hip":
            M.face(((a0, y0, b0), P1, (a0, y0, b1)), r)
            M.face(((a1, y0, b0), (a1, y0, b1), P2), r)
        else:
            for xe in (x0, x1):
                M.face(((xe, y0, z0), (xe, y1 - 0.1, zc), (xe, y0, z1)), "wall")

def house(name, seed):
    """A two- or three-storey house, about 9 x 8 m: walls, a hip or gable roof with eaves, a balcony, units."""
    random.seed(seed)
    M = Mesh()
    w, d = random.uniform(7.5, 10), random.uniform(7, 9.5)
    floors = random.choice([2, 2, 2, 3])
    h = 2.85 * floors + 0.4
    x0, x1, z0, z1 = -w / 2, w / 2, -d / 2, d / 2
    M.box((x0, 0, z0), (x1, h, z1), "wall")
    if random.random() < 0.5:                             # a lower wing with its own roof
        wx = w * random.uniform(0.35, 0.5)
        M.box((x0 - 2.5, 0, z0 + 1), (x0, 3.2, z0 + 1 + d * 0.6), "wall")
        roof(M, x0 - 2.5, x0, z0 + 1, z0 + 1 + d * 0.6, 3.2, "gable", 0.35, 0.35, "roof")
    tin = random.random() < 0.3
    pitch = random.uniform(0.3, 0.5) if not tin else 0.25
    roof(M, x0, x1, z0, z1, h, random.choice(["hip", "gable", "hip"]), pitch, 0.5, "tin" if tin else "roof")
    if not tin and random.random() < 0.5:                 # solar panels on the slope toward +z or -x
        if (z1 - z0) >= (x1 - x0):
            xa, xb = x0 - 0.5 + 0.4, x0 - 0.5 + (w / 2 + 0.5) * 0.8
            ya, yb = h + 0.1 + pitch * 0.4, h + 0.1 + pitch * (w / 2 + 0.5) * 0.8
            M.face(((xa, ya, z0 + 1.2), (xa, ya, z1 - 1.2), (xb, yb, z1 - 1.2), (xb, yb, z0 + 1.2)), "solar")
        else:
            za, zb = z1 + 0.5 - 0.4, z1 + 0.5 - (d / 2 + 0.5) * 0.8
            ya, yb = h + 0.1 + pitch * 0.4, h + 0.1 + pitch * (d / 2 + 0.5) * 0.8
            M.face(((x0 + 1.2, ya, za), (x1 - 1.2, ya, za), (x1 - 1.2, yb, zb), (x0 + 1.2, yb, zb)), "solar")
    for fl in range(1, floors):                           # balconies on the long side
        if random.random() < 0.7:
            bx0 = random.uniform(x0, x0 + w * 0.3); bx1 = bx0 + w * random.uniform(0.4, 0.6)
            y = 2.85 * fl
            M.box((bx0, y, z1), (bx1, y + 0.15, z1 + 1.0), "trim")
            M.box((bx0, y + 0.15, z1 + 0.95), (bx1, y + 1.1, z1 + 1.0), "rail")
            if random.random() < 0.6:
                M.box((bx1 - 1.0, y + 0.15, z1 + 0.1), (bx1 - 0.2, y + 0.75, z1 + 0.45), "unit")
    for k in range(random.randint(1, 3)):                 # air-conditioner units at the foot of the walls
        zz = random.uniform(z0 + 0.5, z1 - 1.3)
        M.box((x1, 0.1, zz), (x1 + 0.32, 0.72, zz + 0.8), "unit")
    M.box((x0 + 0.5, 2.3, z1), (x0 + 2.2, 2.45, z1 + 0.9), "trim")   # the porch canopy
    export(name, [M.obj(name)])

def apartment(name, seed):
    """A small apartment block, 3-5 floors: balconies along the front, a stair core, a parapet, a water tank."""
    random.seed(seed)
    M = Mesh()
    w, d = random.uniform(16, 24), random.uniform(9, 11)
    floors = random.randint(3, 5)
    h = 3.0 * floors + 0.6
    x0, x1, z0, z1 = -w / 2, w / 2, -d / 2, d / 2
    M.box((x0, 0, z0), (x1, h, z1), "wall")
    M.box((x0, h, z0), (x1, h + 0.9, z0 + 0.2), "trim"); M.box((x0, h, z1 - 0.2), (x1, h + 0.9, z1), "trim")
    M.box((x0, h, z0), (x0 + 0.2, h + 0.9, z1), "trim"); M.box((x1 - 0.2, h, z0), (x1, h + 0.9, z1), "trim")
    for fl in range(1, floors):
        y = 3.0 * fl
        M.box((x0, y - 0.1, z1), (x1, y + 0.1, z1 + 1.3), "trim")          # the balcony slab
        M.box((x0, y + 0.1, z1 + 1.2), (x1, y + 1.1, z1 + 1.3), "rail")    # its front
        for k in range(int(w / 3.6)):
            if random.random() < 0.55:
                xx = x0 + 0.6 + k * 3.6
                M.box((xx, y + 0.1, z1 + 0.2), (xx + 0.8, y + 0.7, z1 + 0.55), "unit")
        for k in range(1, int(w / 3.6)):                  # partitions
            xx = x0 + k * 3.6
            M.box((xx - 0.05, y + 0.1, z1), (xx + 0.05, y + 2.6, z1 + 1.2), "trim")
    sx = random.choice([x0 - 2.6, x1])                    # the stair core
    M.box((sx, 0, z0 + 1), (sx + 2.6, h + 1.6, z0 + 4.5), "wall")
    tx, tz = random.uniform(x0 + 2, x1 - 4), random.uniform(z0 + 1.5, z1 - 3)
    M.box((tx, h, tz), (tx + 2.2, h + 0.6, tz + 1.6), "unit")
    M.box((tx + 0.1, h + 0.6, tz + 0.1), (tx + 2.1, h + 2.2, tz + 1.5), "tank")
    export(name, [M.obj(name)])

def block(name, seed, floors):
    """A mid-rise: shops below, offices or flats above, a parapet and the clutter on the roof."""
    random.seed(seed)
    M = Mesh()
    w, d = random.uniform(12, 20), random.uniform(12, 18)
    h = 3.2 * floors + 0.6
    x0, x1, z0, z1 = -w / 2, w / 2, -d / 2, d / 2
    M.box((x0, 0, z0), (x1, h, z1), "wall")
    M.box((x0, 3.6, z0 - 0.5), (x1, 4.1, z0), "trim")                     # the shops' fascia
    M.box((x0, h, z0), (x1, h + 1.0, z0 + 0.25), "trim"); M.box((x0, h, z1 - 0.25), (x1, h + 1.0, z1), "trim")
    M.box((x0, h, z0), (x0 + 0.25, h + 1.0, z1), "trim"); M.box((x1 - 0.25, h, z0), (x1, h + 1.0, z1), "trim")
    for k in range(random.randint(2, 4)):
        bw, bd, bh = random.uniform(1.5, 4), random.uniform(1.5, 4), random.uniform(1.2, 3.5)
        cx, cz = random.uniform(x0 + bw, x1 - bw), random.uniform(z0 + bd, z1 - bd)
        M.box((cx - bw / 2, h, cz - bd / 2), (cx + bw / 2, h + bh, cz + bd / 2), "unit")
    export(name, [M.obj(name)])

# ---- steel and concrete ------------------------------------------------------------------------------------------

def ibeam(M, a, b, w, r):
    """An H-section post or beam: two flanges and a web."""
    a, b = Vector(a), Vector(b)
    t = (b - a).normalized()
    s = t.cross(Vector((0, 0, 1)))
    if s.length < 1e-3:
        s = t.cross(Vector((1, 0, 0)))
    s.normalize()
    for off in (-w / 2, w / 2):
        M.member(a + s * off, b + s * off, 0.03, w, r)
    M.member(a, b, w, 0.03, r)

def mast():
    """The overhead line's portal: two H-section masts on bases, a lattice beam, the arms down to the wires."""
    M = Mesh()
    X2, MID = 4.3, 2.15
    for xm in (MID - 5.95, MID + 5.95):
        M.box((xm - 0.45, -1.5, -0.45), (xm + 0.45, -0.2, 0.45), "conc")
        ibeam(M, (xm, -0.2, 0), (xm, 8.6, 0), 0.32, "steel")
    for y in (7.75, 8.35):                                # the beam: two chords, the lacing between
        M.member((MID - 6.1, y, 0), (MID + 6.1, y, 0), 0.14, 0.14, "steel")
    for k in range(12):
        xa = MID - 5.9 + k * 0.98
        M.member((xa, 7.75, 0), (xa + 0.98, 8.35, 0), 0.06, 0.06, "steel")
    for xw in (0.0, X2):                                  # hangers, arms, the register arms
        M.member((xw, 7.75, 0), (xw, 6.1, 0), 0.07, 0.07, "steel")
        M.member((xw - 0.6, 6.15, 0), (xw + 0.6, 6.15, 0), 0.06, 0.06, "steel")
        M.member((xw, 6.1, 0), (xw + 0.2, 5.2, 0.3), 0.04, 0.04, "steel")
    export("mast", [M.obj("mast")])

def utility_pole(name, transformer):
    M = Mesh()
    M.member((0, 0, 0), (0, 11.2, 0), 0.34, 0.34, "conc", sides=8)
    M.box((-0.95, 9.9, -0.06), (0.95, 10.05, 0.06), "steel")
    M.box((-0.7, 8.6, -0.06), (0.7, 8.72, 0.06), "steel")
    for x in (-0.8, 0.0, 0.8):
        M.member((x, 10.05, 0), (x, 10.3, 0), 0.08, 0.08, "unit", sides=6)
    if transformer:
        M.member((0.45, 7.0, 0), (0.45, 8.2, 0), 0.55, 0.55, "unit", sides=8)
    M.member((0, 6.0, 0), (0.35, 6.0, 0), 0.05, 0.05, "steel")
    export(name, [M.obj(name)])

def truss_bridge(length=120.0):
    """A through truss, Warren with verticals, 10 m panels, gusset plates at the joints; its deck and piers."""
    M = Mesh()
    MID = 2.15
    TOP = 9.0
    xs = (MID - 5.5, MID + 5.5)
    nodes = [10.0 * k for k in range(int(length / 10) + 1)]
    n = len(nodes)
    def h_member(a, b, w):
        M.member(a, b, w, w * 1.2, "steel")
    for xt in xs:
        h_member((xt, 0.0, 0), (xt, 0.0, length), 0.75)
        h_member((xt, TOP, 10), (xt, TOP, length - 10), 0.75)
        h_member((xt, 0.0, 0), (xt, TOP, 10), 0.75)
        h_member((xt, 0.0, length), (xt, TOP, length - 10), 0.75)
        for k in range(1, n - 1):
            zn = nodes[k]
            h_member((xt, 0.0, zn), (xt, TOP, zn), 0.42)
            if k < n - 2:
                up = (k % 2 == 1) if k < n // 2 else (k % 2 == 0)
                a, b = ((zn, 0.0), (zn + 10, TOP)) if up else ((zn, TOP), (zn + 10, 0.0))
                h_member((xt, a[1], a[0]), (xt, b[1], b[0]), 0.48)
            for y in (0.0, TOP):                           # gusset plates
                M.box((xt - 0.45, y - 0.9, zn - 1.0), (xt + 0.45, y + 0.9, zn + 1.0), "steel")
    for k in range(1, n - 1):
        zn = nodes[k]
        M.member((xs[0], TOP, zn), (xs[1], TOP, zn), 0.4, 0.6, "steel")
        M.member((xs[0], TOP - 1.0, zn), (xs[0] + 1.2, TOP, zn), 0.2, 0.2, "steel")   # knee braces
        M.member((xs[1], TOP - 1.0, zn), (xs[1] - 1.2, TOP, zn), 0.2, 0.2, "steel")
        if k < n - 2:
            M.member((xs[0], TOP + 0.1, zn), (xs[1], TOP + 0.1, zn + 10), 0.22, 0.22, "steel")
            M.member((xs[1], TOP + 0.1, zn), (xs[0], TOP + 0.1, zn + 10), 0.22, 0.22, "steel")
        M.member((xs[0], -0.8, zn), (xs[1], -0.8, zn), 0.4, 0.8, "steel")              # floor beams
    for zp in (10, length - 10):
        M.member((xs[0], TOP - 1.4, zp), (xs[1], TOP - 1.4, zp), 0.4, 1.2, "steel")
    M.box((MID - 6.0, -2.4, 0), (MID + 6.0, -0.35, length), "conc", bottom=True)
    for zp in [40.0 * k for k in range(1, int(length / 40))]:
        M.box((MID - 3, -9.0, zp - 1.6), (MID + 3, -2.4, zp + 1.6), "conc")
    for zb, s in ((0.0, -1), (length, 1)):
        M.box((MID - 7, -4.0, zb - (6 if s < 0 else 0)), (MID + 7, -0.35, zb + (6 if s > 0 else 0)), "conc")
    export("truss_bridge", [M.obj("truss_bridge")])

def main():
    random.seed(1964)
    reset()
    objs = [train_car("n700_nose", True)]
    export("n700_nose", objs)
    reset(); export("n700_mid", [train_car("n700_mid", False)])
    reset(); export("pantograph", [pantograph()])
    for i in range(5):
        reset(); tree_broadleaf("tree_broad_%d" % i, 11 + i * 7, random.uniform(8, 10.5), random.uniform(3.0, 4.0))
    for i in range(3):
        reset(); tree_cedar("tree_cedar_%d" % i, 101 + i * 5, 15 + i * 2)
    for i in range(3):
        reset(); tree_pine("tree_pine_%d" % i, 201 + i * 9, 8 + i)
    for i in range(2):
        reset(); tree_bamboo("tree_bamboo_%d" % i, 301 + i * 4, 11 + i * 2)
    for sp in ("broad", "cedar", "pine", "bamboo"):
        reset(); tree_far("tree_%s_far" % sp, 700 + len(sp), sp)
    for i in range(8):
        reset(); house("house_%d" % i, 401 + i * 13)
    for i in range(4):
        reset(); apartment("apartment_%d" % i, 501 + i * 17)
    for i, fl in enumerate((5, 7, 9, 12)):
        reset(); block("block_%d" % i, 601 + i * 19, fl)
    reset(); mast()
    reset(); utility_pole("pole", False)
    reset(); utility_pole("pole_tr", True)
    reset(); truss_bridge()
    with open(os.path.join(OUT, "counts.txt"), "w") as f:
        for k, v in sorted(COUNTS.items()):
            f.write("%s %d\n" % (k, v))

main()
