"""The night extension's models (the train pulling into a station after dark): the station, its big LED board, a
street light, the drone, and the stadium blocked out until the audience design's arrives. Built from numbers in
Blender like build_models.py (whose Mesh and export it borrows), exported as triangle meshes (.obj).

    blender -b --factory-startup -P build_night_models.py -- OUTDIR

The film's coordinates (metres, y up, the line along +z, our line at x = 0, the other at x = 4.3). The station is
built from its platforms' start (z = 0); the stadium in its own frame (see stadium()).
"""
import math
import os
import random
import sys

sys.argv = [a for a in sys.argv]                          # build_models reads OUTDIR after "--" too
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
src = open(os.path.join(HERE, "build_models.py")).read()
src = src[:src.index("def main():")]                       # the helpers only, not its main
exec(compile(src, "build_models.py", "exec"))

X2, MID = 4.3, 2.15
PLATFORM = 200.0                                          # the stretch the cab sees (the train overhangs behind)
PH = 1.25                                                 # the platform's top above the rails

def station():
    """Two side platforms with canopies, fences and light tubes; the station building beyond the right one."""
    M = Mesh()
    for side, (xa, xb) in ((-1, (-10.5, -1.75)), (1, (X2 + 1.75, X2 + 10.5))):
        M.box((xa, -1.6, 0.0), (xb, PH, PLATFORM), "platform")
        edge = xb if side < 0 else xa
        xf = edge + (-0.55 if side < 0 else 0.55)          # the fence back from the edge, gaps at the doors
        z = 4.0
        while z < PLATFORM - 6:
            M.box((xf - 0.05, PH, z), (xf + 0.05, PH + 1.25, z + 18.5), "fence")
            M.box((xf - 0.09, PH + 1.2, z), (xf + 0.09, PH + 1.32, z + 18.5), "steel")
            z += 25.0
        xc = (xa + xb) / 2
        for zc in [7.5 + 15 * k for k in range(int((PLATFORM - 10) / 15) + 1)]:
            M.member((xc, PH, zc), (xc, 7.6, zc), 0.45, 0.45, "steel", sides=8)
            M.member((xc, 7.0, zc), (xa - 0.6, 7.75, zc), 0.2, 0.3, "steel")
            M.member((xc, 7.0, zc), (xb + 0.6, 7.75, zc), 0.2, 0.3, "steel")
        M.box((xa - 0.8, 7.75, -2.0), (xb + 0.8, 8.15, PLATFORM + 2.0), "canopy", bottom=True)
        for xt in (xa + 2.2, xb - 2.2):                   # the light tubes under it, in two rows
            for zt in [2.0 + 5.0 * k for k in range(int(PLATFORM / 5.0))]:
                M.box((xt - 0.08, 7.62, zt), (xt + 0.08, 7.74, zt + 3.2), "tube")
        for zs in [30.0 + 60 * k for k in range(3)]:      # car-position signs hanging over the edge
            M.box((edge - 0.6 * side - 0.3, 5.2, zs), (edge - 0.6 * side + 0.3, 5.7, zs + 0.06), "signsmall")
    # the building: a concourse with a glass front toward the platforms, offices above
    bx0, bx1 = X2 + 12.0, X2 + 46.0
    M.box((bx0, -1.6, 40.0), (bx1, 16.0, 170.0), "buildwall")
    M.box((bx0 - 0.4, 16.0, 38.0), (bx1 + 0.4, 17.0, 172.0), "trim")
    for k in range(3):
        cx = bx0 + 6 + k * 11
        M.box((cx, 17.0, 70 + 30 * k), (cx + 5, 19.5, 76 + 30 * k), "unit")
    M.box((bx0 - 2.0, 10.5, 70.0), (bx0 - 0.4, 14.0, 140.0), "namesign")   # the station's name on its side
    return M.obj("station")

def led_board():
    """The big platform board: a dark frame, LED faces both sides (the prototype lights the dots), on two hangers."""
    M = Mesh()
    w, h, d = 9.6, 2.9, 0.35
    M.box((-w / 2 - 0.15, -0.15, -d / 2 - 0.05), (w / 2 + 0.15, h + 0.15, d / 2 + 0.05), "frame", bottom=True)
    M.face(((-w / 2, 0, d / 2 + 0.06), (w / 2, 0, d / 2 + 0.06), (w / 2, h, d / 2 + 0.06), (-w / 2, h, d / 2 + 0.06)), "led")
    M.face(((w / 2, 0, -d / 2 - 0.06), (-w / 2, 0, -d / 2 - 0.06), (-w / 2, h, -d / 2 - 0.06), (w / 2, h, -d / 2 - 0.06)), "led")
    for x in (-w / 3, w / 3):
        M.member((x, h + 0.15, 0), (x, h + 1.2, 0), 0.08, 0.08, "steel")
    return M.obj("led_board")

def street_light():
    M = Mesh()
    M.member((0, 0, 0), (0, 7.5, 0), 0.16, 0.16, "steel", sides=6)
    M.member((0, 7.3, 0), (1.6, 7.6, 0), 0.08, 0.08, "steel")
    M.box((1.3, 7.42, -0.18), (2.1, 7.58, 0.18), "steel")
    M.box((1.35, 7.36, -0.14), (2.05, 7.42, 0.14), "lamp")
    return M.obj("street_light")

def drone():
    """A quadcopter about 0.8 m across, nose toward +z: body, arms, motors, rotors, legs, a gimbal, its lights."""
    M = Mesh()
    M.box((-0.13, -0.05, -0.18), (0.13, 0.06, 0.18), "dronebody", bottom=True)
    M.box((-0.08, 0.06, -0.12), (0.08, 0.1, 0.1), "dronebody")
    for sx, sz in ((-1, 1), (1, 1), (-1, -1), (1, -1)):
        tip = (0.3 * sx, 0.0, 0.3 * sz)
        M.member((0.05 * sx, 0.0, 0.05 * sz), tip, 0.04, 0.03, "dronebody")
        M.member((tip[0], -0.03, tip[2]), (tip[0], 0.05, tip[2]), 0.06, 0.06, "dronebody", sides=8)
        M.member((tip[0], 0.055, tip[2]), (tip[0], 0.065, tip[2]), 0.36, 0.36, "rotor", sides=12)
        M.member((0.12 * sx, -0.05, 0.1 * sz), (0.16 * sx, -0.16, 0.12 * sz), 0.02, 0.02, "dronebody")
    M.member((0, -0.05, 0.14), (0, -0.1, 0.16), 0.03, 0.03, "dronebody")
    M.box((-0.04, -0.15, 0.12), (0.04, -0.08, 0.2), "dronebody")
    M.box((-0.33, -0.02, 0.27), (-0.27, 0.02, 0.33), "navred")
    M.box((0.27, -0.02, 0.27), (0.33, 0.02, 0.33), "navgreen")
    M.box((-0.025, 0.1, -0.12), (0.025, 0.13, -0.07), "strobe")
    return M.obj("drone")

def stadium():
    """The stadium blocked out to the audience design's level 9 (prototypes/audience/rooms.py): an oval bowl
    round a 150 x 110 m pitch, 34 rows rising a metre each and 1.8 m deep, open at the far end for a screen 100 m
    wide (centre 36 m up), the stage under it. Built in the film's axes with the audience's frame mapped as
    x' = X (their right), y' = Z (up), z' = Y (toward the screen); the prototype turns it a quarter so the screen
    faces away from the station."""
    M = Mesh()
    A, B, rows = 79.0, 59.0, 34
    n = 64
    ell = lambda k: (A + k * 1.8, B + k * 1.8)
    def pt(k, a, y):
        rx, ry = ell(k)
        return (rx * math.cos(a), y, ry * math.sin(a))
    for j in range(n):
        a0, a1 = 2 * math.pi * j / n, 2 * math.pi * (j + 1) / n
        if math.sin((a0 + a1) / 2) > 0.82:
            continue
        for k in range(0, rows, 2):                       # the rake, two rows to a band
            y0, y1 = 1.0 + k, 1.0 + k + 2
            M.face((pt(k, a0, y0), pt(k + 2, a0, y1), pt(k + 2, a1, y1), pt(k, a1, y0)), "tier")
        rx, ry = ell(rows)
        M.face((pt(rows, a0, 0.0), pt(rows, a1, 0.0), pt(rows, a1, rows + 1.0), pt(rows, a0, rows + 1.0)), "outer")
        M.face((pt(0, a0, 0.0), pt(0, a0, 1.0), pt(0, a1, 1.0), pt(0, a1, 0.0)), "outer")
        if j % 4 == 2:                                    # light towers on the rim (none where the flight crosses)
            x, y, z = pt(rows, a0, rows + 1.0)
            M.member((x, y, z), (x * 1.02, y + 14, z * 1.02), 0.8, 0.8, "outer")
            M.box((x * 1.02 - 2.5, y + 14, z * 1.02 - 1.0), (x * 1.02 + 2.5, y + 17, z * 1.02 + 1.0), "flood")
    M.box((-60, 0.0, B - 10), (60, 3.0, B + 29), "stage")
    M.box((-52, 7.0, B + 29.6), (52, 65.0, B + 31.0), "outer")                   # the screen's back
    M.face(((-50, 7.875, B + 29.5), (50, 7.875, B + 29.5), (50, 64.125, B + 29.5), (-50, 64.125, B + 29.5)), "screen")
    M.box((-110, -0.2, -130), (110, 0.0, 130), "pitch")
    return M.obj("stadium")

BLIMP_SCREEN = dict(c=(4.0, 2.0, -24.5), w=33.0, h=18.6, tilt=0.26)   # centre, size, tilted down (radians)

def blimp():
    """An advertising airship (our own, after the look of Blade Runner's): a hundred and ten metres of dark lobed
    hull, wider than tall, pointed at both ends; gas-cell lobes bulging along its upper flanks; plates and boxes
    all over; antenna masts with cross-bars and dishes bristling from both ends; engine pods and a gondola under it;
    a huge rounded screen in a raised frame on a box out from its -z flank, tilted down to the street. The prototype
    lights it: the picture, the frame's glow, hundreds of running lights, searchlights."""
    random.seed(1982)
    M = Mesh()
    L, RW, RH = 55.0, 21.0, 15.0
    nu, nv = 56, 24
    def hull(u, v):                                       # u along (-1..1), v round
        base = max(0.0, 1 - abs(u) ** 2.2) ** 0.55
        side, up = math.cos(v), math.sin(v)
        lobe = (0.5 + 0.5 * math.cos(u * math.pi * 6.5)) ** 2 * max(0.0, up + 0.35) * abs(side) ** 0.5
        r = 1 + 0.16 * lobe - 0.10 * max(0.0, -up) ** 2
        return (L * u, RH * base * r * up * (1.15 if up > 0 else 0.85), RW * base * r * side)
    for i in range(nu):
        u0, u1 = -1 + 2 * i / nu, -1 + 2 * (i + 1) / nu
        for j in range(nv):
            v0, v1 = 2 * math.pi * j / nv, 2 * math.pi * (j + 1) / nv
            M.face((hull(u0, v0), hull(u1, v0), hull(u1, v1), hull(u0, v1)), "hull")
    for k in range(260):                                  # plates, boxes, vents
        u = random.uniform(-0.8, 0.8); v = random.uniform(0, 2 * math.pi)
        x, y, z = hull(u, v)
        n = Vector((0, y, z)).normalized()
        s = random.uniform(0.8, 4.5); h = random.uniform(0.3, 2.2)
        c = Vector((x, y, z)) + n * h * 0.4
        t1 = Vector((1, 0, 0)); t2 = n.cross(t1)
        pts = [c + t1 * a * s + t2 * b * s * random.uniform(0.4, 0.9) + n * e * h * 0.5
               for a in (-1, 1) for b in (-1, 1) for e in (-1, 1)]
        for f in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)):
            M.face(tuple(tuple(pts[i]) for i in f), "greeble")
    for end in (-1, 1):                                   # antenna masts bristling from both ends
        for k in range(9):
            a = random.uniform(-1.2, 1.2); b = random.uniform(-0.9, 0.9)
            base = Vector(hull(end * random.uniform(0.75, 0.93), random.uniform(0, 2 * math.pi)))
            d = Vector((end * math.cos(a) * math.cos(b), math.sin(b), math.sin(a) * math.cos(b))).normalized()
            hgt = random.uniform(8, 22)
            tip = base + d * hgt
            M.member(tuple(base), tuple(tip), 0.3, 0.3, "mast", sides=5)
            for q in (0.45, 0.75):
                c = base + d * hgt * q
                side = d.cross(Vector((0, 1, 0)))
                if side.length < 1e-3:
                    side = Vector((1, 0, 0))
                side = side.normalized() * random.uniform(1.0, 2.5)
                M.member(tuple(c - side), tuple(c + side), 0.18, 0.18, "mast")
            if random.random() < 0.5:                     # a dish
                c = base + d * hgt * random.uniform(0.3, 0.6)
                M.member(tuple(c), tuple(c + d.cross(Vector((0, 0, 1))).normalized() * 0.3 + Vector((0, 0.01, 0))),
                         2.2, 2.2, "mast", sides=10)
    for x in (-38, 38):                                   # engine pods
        for zz in (-14, 14):
            M.member((x - 5, -RH * 0.55, zz), (x + 5, -RH * 0.55, zz), 3.2, 3.2, "greeble", sides=10)
    M.box((-16, -RH - 4.0, -5), (16, -RH + 2.0, 5), "greeble", bottom=True)        # the gondola
    # the screen: a box out from the -z flank, the screen in a raised rounded frame, tilted down
    sc = BLIMP_SCREEN
    cx, cy, cz = sc["c"]
    w, h, t = sc["w"], sc["h"], sc["tilt"]
    nrm_ = Vector((0, -math.sin(t), -math.cos(t)))         # facing out and a little down
    upv = Vector((0, math.cos(t), -math.sin(t)))
    rt = Vector((1, 0, 0))
    C = Vector((cx, cy, cz))
    def P(a, b, off=0.0):
        return tuple(C + rt * a + upv * b + nrm_ * off)
    def rounded(w, h, r, n=6):
        pts = []
        for ccx, ccy, a0 in ((w / 2 - r, h / 2 - r, 0), (-w / 2 + r, h / 2 - r, 90), (-w / 2 + r, -h / 2 + r, 180),
                             (w / 2 - r, -h / 2 + r, 270)):
            for k in range(n + 1):
                a = math.radians(a0 + 90 * k / n)
                pts.append((ccx + r * math.cos(a), ccy + r * math.sin(a)))
        return pts
    inner, outer = rounded(w, h, 3.5), rounded(w + 4.0, h + 4.0, 5.0)
    M.face([P(a, b) for a, b in inner], "bscreen")
    for (a, b), (c, d) in zip(zip(inner, inner[1:] + inner[:1]), zip(outer, outer[1:] + outer[:1])):
        M.face((P(a[0], a[1], 0.6), P(b[0], b[1], 0.6), P(d[0], d[1], 0.2), P(c[0], c[1], 0.2)), "bframe")
        M.face((P(a[0], a[1], 0.0), P(b[0], b[1], 0.0), P(b[0], b[1], 0.6), P(a[0], a[1], 0.6)), "greeble")
    # the box behind it, back to the hull
    hw, hh = w / 2 + 3.0, h / 2 + 3.0
    corners = [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]
    back = [tuple(Vector(P(a, b, -9.0))) for a, b in corners]
    front = [P(a, b, 0.0) for a, b in corners]
    for k in range(4):
        M.face((front[k], front[(k + 1) % 4], back[(k + 1) % 4], back[k]), "greeble")
    for a in (-hw, 0, hw):                                # struts down to the hull's belly
        M.member(P(a, -hh, -4.0), (cx + a * 0.6, -RH * 0.6, -RW * 0.5), 0.5, 0.5, "mast")
    return M.obj("blimp")

def main():
    random.seed(10)
    for f in (station, led_board, street_light, drone, stadium, blimp):
        reset()
        ob = f()
        export(ob.name, [ob])
    with open(os.path.join(OUT, "counts_night.txt"), "w") as fh:
        for k, v in sorted(COUNTS.items()):
            fh.write("%s %d\n" % (k, v))

main()
