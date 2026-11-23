#!/usr/bin/env python3
"""Design prototype: the dive's new ending. The portal, its glow, the whiteout and the caption
go; where the portal stood, the diver finds a volume knob: a big, chunky old amp dial on a rock pillar, glowing
faintly in the dark, its scale printed 0 to 10, set to 9. The diver comes to a stop over it; one decisive click
turns it to 10, and on that frame the dive ends: a hard cut to the bullet train (no hold).

    python knob.py dive          render the dive again without the portal, caption and whiteout, the camera
                                 coming to a stop over the knob (the dive's own reference, natively) into WORK
    python knob.py OUT.mp4 [--small]

The knob is ray-cast here in floating point (a design, not the build): the dial plate, the knob with its knurled
edge and pointer, the pillar under it; lit only by its own glow and a little of the water's light from above; hazed
by the dive's fog; the printed scale glowing. The dive's frames from the HD video rendered again by the reference
(with its depths) for the changed script; the train's first frames from segments/train/video_hd.rgb after the cut.
"""
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))

def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


DEEP = os.path.abspath(os.path.join(HERE, "..", "segments", "deep"))
WORK = _work("knob")
TRAIN = os.path.join(HERE, "..", "segments", "train", "video_hd.rgb")
W, H, F = 1536, 768, 800

# the timing: the camera as the dive flies it until STOP0, slowing to a stop at the frame of the click; the dial
# at 9 until CLICK, turning over two frames, on 10 at LAST, the dive's last frame
STOP0, STOP1, LAST = 290, 308, 352
# after the click: the dial powers up (frames 332..349), then lightning (350, 351) and the flash (352)
POWER0, POWER1, BOLT = 332, 349, 350
# the camera eases to a stop at the dive's frame 299's place (about 8 units from the dial)
STOP_AT = 299
# where the portal stood (its middle, from the script: x, y, z units): the dial's middle, on its pillar
KNOB = np.array([-1.66, -109.22, 321.9])
PLATE_R, PLATE_T, KNOB_R, KNOB_H = 4.6, 0.45, 1.8, 1.0
ROCK_R = 6.4
SWEEP = 300.0                       # the scale's sweep (degrees), 0 at -150 from the top, 10 at +150


def smooth(e0, e1, x):
    t = min(max((x - e0) / (e1 - e0), 0.0), 1.0)
    return t * t * (3 - 2 * t)


def script_lines():
    return open(os.path.join(DEEP, "data", "script.txt")).read().splitlines()


def edited_script(out):
    """The script without the portal, caption and whiteout, the camera slowing to a stop: from STOP0 on, frame n
    shows the camera of frame STOP0 + s(n), s easing from 1 a frame to none at LAST."""
    lines = script_lines()
    # past the dive's 309 frames: frame 308's numbers, the water's phases going on (as deep.rs works them out)
    while len(lines) < LAST + 1:
        f = len(lines)
        v = list(map(int, lines[308].split()))
        v[17:20] = [(f * 1353 // 1000) % 60, (60 - (f * 1035 // 1000) % 60) % 60, (f * 796 // 1000) % 60]
        v[20:22] = [(12 * f) % 6000, (-8 * f) % 6000]
        v[22] = (f * 2387 // 1000) % 60
        v[45] = round(-23.873 * f / 12) % 60
        lines.append(" ".join(map(str, v)))
    span = STOP1 - STOP0
    outl = []
    for n, line in enumerate(lines):
        v = list(map(int, line.split()))
        if n >= STOP0:
            k = min(n, STOP1) - STOP0
            eff = STOP0 + (STOP_AT - STOP0) * (2 * k / span - (k / span) ** 2)
            a, b = int(math.floor(eff)), eff - math.floor(eff)
            va = list(map(int, lines[a].split()))
            vb = list(map(int, lines[min(a + 1, len(lines) - 1)].split()))
            # the camera's place between the two frames (whole cards: x hundredths; y units + hundredths; z units +
            # tenths), its vectors as the earlier frame's (they hold still here)
            x = round(va[1] + (vb[1] - va[1]) * b)
            y = (va[2] * 100 + va[3]) + ((vb[2] * 100 + vb[3]) - (va[2] * 100 + va[3])) * b
            z = (va[4] * 10 + va[5]) + ((vb[4] * 10 + vb[5]) - (va[4] * 10 + va[5])) * b
            y, z = round(y), round(z)
            v[1], v[2], v[3], v[4], v[5] = x, y // 100, y % 100, z // 10, z % 10
            v[6:15] = va[6:15]
            v[24:33] = va[24:33]
        # no white frames, no caption, no portal, no whiteout
        v[0] = 0
        if len(v) > 43:
            v[36], v[37], v[43] = -1, 0, 0
        outl.append(" ".join(map(str, v)))
    open(out, "w").write("\n".join(outl) + "\n")


def dive():
    """The dive again for the edited script (the reference, natively), with its depths."""
    os.makedirs(WORK, exist_ok=True)
    edited_script(WORK + "/script.txt")
    src = open(os.path.join(DEEP, "reference.rs")).read()
    src = src.replace('#[path = "../../demokit/src/lib.rs"]', '#[path = "%s/../../demokit/src/lib.rs"]' % DEEP)
    for m in ("reference_hd.rs", "reference_orca.rs"):
        src = src.replace('#[path = "%s"]' % m, '#[path = "%s/%s"]' % (DEEP, m))
    src = src.replace("        hd::frame(&scene, &hd_data, &trig, &mut hd_rgb);",
                      "        hd::frame(&scene, &hd_data, &trig, &mut hd_rgb);\n        { use std::io::Write as _; let mut df = std::fs::OpenOptions::new().append(true).create(true).open(\"%s/depth.bin\").unwrap(); df.write_all(&under_depth.iter().flat_map(|&d| (d as i16).to_le_bytes()).collect::<Vec<u8>>()).unwrap(); }" % WORK)
    open(WORK + "/reference_knob.rs", "w").write(src)
    if os.path.exists(WORK + "/depth.bin"):
        os.remove(WORK + "/depth.bin")
    subprocess.run(["rustc", "-O", "-o", WORK + "/reference_knob", WORK + "/reference_knob.rs"], check=True)
    data = os.path.join(DEEP, "data")
    swarm = os.path.abspath(os.path.join(DEEP, "..", "swarm", "data", "tables.txt"))
    with open(WORK + "/both.rgb", "wb") as fh:
        subprocess.run([WORK + "/reference_knob", data + "/map.txt", WORK + "/script.txt", data + "/font3x5.txt", data + "/hd.txt", swarm], stdout=fh, check=True)
    with open(WORK + "/both.rgb", "rb") as fh, open(WORK + "/video_hd.rgb", "wb") as out:
        fh.seek((LAST + 1) * 192 * 96 * 3)
        while True:
            b = fh.read(1 << 24)
            if not b:
                break
            out.write(b)
    os.remove(WORK + "/both.rgb")


def cameras(path):
    cams = []
    for line in open(path):
        v = list(map(int, line.split()))
        pos = np.array([v[1] / 100, v[2] + v[3] / 100, v[4] + v[5] / 10])
        f, r, u = (np.array(v[6:9]) / 300, np.array(v[9:12]) / 300, np.array(v[12:15]) / 300)
        light = v[15] / 32
        far = np.array([1, 3, 8]) + (np.array([6, 40, 62]) - np.array([1, 3, 8])) * light
        cams.append((pos, f, r, u, far))
    return cams


def haze(d):
    return np.clip(0.85 * (np.maximum(d, 0) / 70) ** 0.8, 0, 1)


def noise(size, cells, seed):
    """Smooth value noise 0..1: a random grid of cells x cells, enlarged smoothly."""
    rng = np.random.default_rng(seed)
    g = Image.fromarray((rng.random((cells, cells)) * 255).astype(np.uint8))
    return np.asarray(g.resize((size, size), Image.BICUBIC)).astype(float) / 255


def fbm(size, seed, octaves=((5, 0.5), (11, 0.25), (23, 0.14), (53, 0.07), (127, 0.04)), warp=0.06):
    """Fractal noise 0..1, its domain warped so nothing in it repeats or lines up."""
    wx, wy = noise(size, 7, seed + 100), noise(size, 7, seed + 200)
    yy, xx = np.mgrid[0:size, 0:size]
    xi = np.clip((xx + (wx - 0.5) * warp * size * 2).astype(int), 0, size - 1)
    yi = np.clip((yy + (wy - 0.5) * warp * size * 2).astype(int), 0, size - 1)
    out = np.zeros((size, size))
    for k, (cells, amp) in enumerate(octaves):
        out += amp * noise(size, cells, seed + k)[yi, xi]
    return (out - out.min()) / (out.max() - out.min())


def blur(a, r):
    """About a Gaussian of radius r: three box blurs each way."""
    k = max(1, int(round(r)))
    out = a.astype(float)
    for _ in range(3):
        for ax in (0, 1):
            c = np.cumsum(np.pad(out, [(k + 1, k) if i == ax else (0, 0) for i in range(2)], mode="edge"), axis=ax)
            hi = np.take(c, np.arange(2 * k + 1, c.shape[ax]), axis=ax)
            lo = np.take(c, np.arange(0, c.shape[ax] - 2 * k - 1), axis=ax)
            out = (hi - lo) / (2 * k + 1)
    return out


def surface(size, seed, crust_amount, barnacles, dents, scratches, rng_rim=None):
    """A worn, fouled metal surface as maps (size x size): its height (relief: crust standing proud, dents, pits,
    scratches, barnacle clusters with their craters), the cavity (how far below what's round it: crevices dark), and
    how crusted, rusted and green each bit is."""
    rng = np.random.default_rng(seed)
    crustn = fbm(size, seed + 1)
    crust = np.clip((crustn - (1 - crust_amount)) * 4, 0, 1)
    rust = np.clip((fbm(size, seed + 2) - 0.55) * 3.5, 0, 1)
    verd = np.clip((fbm(size, seed + 3) - 0.6) * 3.5, 0, 1)
    h = 0.55 * crust * (0.6 + 0.8 * fbm(size, seed + 4, ((31, 0.6), (97, 0.4)), 0.02))
    # pits: small round hollows, uneven
    pit = fbm(size, seed + 5, ((90, 0.6), (200, 0.4)), 0.03)
    h -= 0.35 * np.clip((pit - 0.6) * 5, 0, 1) * np.clip((fbm(size, seed + 6, ((4, 0.6), (9, 0.4)), 0.1) - 0.55) * 4, 0, 1)
    yy, xx = np.mgrid[0:size, 0:size]
    # dents: broad shallow hollows
    for _ in range(dents):
        cx, cy, r = rng.uniform(0, size), rng.uniform(0, size), rng.uniform(0.03, 0.09) * size
        h -= 0.5 * np.exp(-((xx - cx) ** 2 + (yy - cy) ** 2) / (2 * r * r))
    # scratches: thin gouges, every which way
    sc = Image.new("L", (size, size), 0)
    dr = ImageDraw.Draw(sc)
    for _ in range(scratches):
        x0, y0 = rng.uniform(0, size), rng.uniform(0, size)
        a = rng.uniform(0, math.pi)
        ln = rng.uniform(0.05, 0.35) * size
        pts = [(x0, y0)]
        for k in range(6):
            a += rng.uniform(-0.15, 0.15)
            x0 += math.cos(a) * ln / 6
            y0 += math.sin(a) * ln / 6
            pts.append((x0, y0))
        dr.line(pts, fill=int(rng.uniform(120, 255)), width=int(rng.integers(1, 4)))
    scr = np.asarray(sc).astype(float) / 255
    h -= 0.3 * scr
    # barnacles: uneven clusters, each a cone with a crater
    barn = np.zeros((size, size))
    for _ in range(barnacles):
        cx, cy = (rng_rim(rng) if rng_rim else (rng.uniform(0, size), rng.uniform(0, size)))
        for _ in range(int(rng.integers(2, 9))):
            bx, by = cx + rng.normal(0, 0.025 * size), cy + rng.normal(0, 0.025 * size)
            br = rng.uniform(0.008, 0.03) * size
            d = np.hypot(xx - bx, yy - by) / br
            cone = np.clip(1 - d, 0, 1) * 0.9 - 0.5 * np.clip(1 - d / 0.4, 0, 1)
            barn = np.maximum(barn, cone)
    h += barn
    cav = np.clip((blur(h, size / 90) - h) * 2, 0, 0.8)
    gy, gx = np.gradient(h)
    return dict(h=h, gx=gx * size / 40, gz=-gy * size / 40, cav=cav, crust=crust, rust=rust, verd=verd, barn=np.clip(barn * 2, 0, 1), scr=scr)


def dial_face(size=1024):
    """The plate's printed face, looking down on it, the top away from the diver: ticks and numerals 0..10 round
    the knob (ink 0..1, and which number each bit belongs to, 0..10, -1 none), the printing worn away in patches
    (9 and 10 kept), and the surfaces' maps: the plate's, the skirt's, the knob's top and side, the rock's top."""
    img = Image.new("L", (size, size), 0)
    idx = Image.new("L", (size, size), 0)
    dr, di = ImageDraw.Draw(img), ImageDraw.Draw(idx)
    c = size / 2
    sc = size / 2 / PLATE_R
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Futura.ttc", int(0.62 * sc))
    except OSError:
        font = ImageFont.load_default()
    for i in range(0, 101):
        v = i / 10
        a = math.radians(-SWEEP / 2 + SWEEP * v / 10)
        major = i % 10 == 0
        r0, r1 = (2.45 if major else 2.6) * sc, (3.05 if major else 2.85) * sc
        dx, dy = math.sin(a), -math.cos(a)
        width = int(0.07 * sc if major else 0.035 * sc)
        dr.line([(c + r0 * dx, c + r0 * dy), (c + r1 * dx, c + r1 * dy)], fill=255 if major else 150, width=width)
        di.line([(c + r0 * dx, c + r0 * dy), (c + r1 * dx, c + r1 * dy)], fill=1 + i // 10, width=width + 2)
        if major:
            rt = 3.7 * sc
            dr.text((c + rt * dx, c + rt * dy), str(i // 10), fill=255, font=font, anchor="mm")
            di.text((c + rt * dx, c + rt * dy), str(i // 10), fill=1 + i // 10, font=font, anchor="mm", stroke_width=4, stroke_fill=1 + i // 10)
    ink = np.asarray(img).astype(float) / 255
    num = np.asarray(idx).astype(int) - 1
    keep = np.clip(blur((num >= 9).astype(float) * 255, 30) / 50, 0, 1)
    rim = lambda rng: (c + rng.uniform(3.9, 4.6) * sc * math.sin(t := rng.uniform(0, 2 * math.pi)), c - rng.uniform(3.9, 4.6) * sc * math.cos(t))
    plate = surface(size, 11, 0.38, 9, 6, 40, rim)
    # the printing worn: gone in patches and where crust or barnacles are, but not on 9 and 10
    wear = np.clip((fbm(size, 17) - 0.45) * 3, 0, 1)
    plate["crust"] *= 1 - keep
    plate["barn"] *= 1 - keep
    inkv = ink * (1 - np.maximum(wear, plate["crust"]) * (1 - keep)) * (1 - plate["barn"])
    return dict(ink=inkv, num=num, plate=plate, skirt=surface(512, 21, 0.45, 5, 3, 12), top=surface(512, 31, 0.5, 2, 4, 18),
                side=surface(512, 41, 0.3, 0, 2, 25), rock=surface(1024, 51, 0.6, 14, 10, 30), rockside=fbm(1024, 61),
                weed=np.clip((fbm(1024, 71) - 0.62) * 4, 0, 1))


FACE = None


def knob_layer(frame, depth, cam, value, n, wobble=0.0, power=0.0, shake=(0.0, 0.0)):
    """Ray-cast the knob into the frame: its pixels, lit by the water's light from above and its own glow, with
    relief (the maps' normals and crevices), hazed, hidden where the dive's seabed is nearer."""
    global FACE
    if FACE is None:
        FACE = dial_face()
    pos, f, r, u, far = cam
    # the dial's own frame: up, and its top toward the far side (away from the diver along the ground)
    fwd = np.array([f[0], 0.0, f[2]]); fwd /= np.linalg.norm(fwd)
    side = np.cross(np.array([0.0, 1.0, 0.0]), fwd)
    up = np.array([0.0, 1.0, 0.0])
    SS = 2
    ys, xs = np.mgrid[0:H * SS, 0:W * SS]
    gx = ((xs + 0.5) / SS - W / 2) / F
    gy = -((ys + 0.5) / SS - H / 2) / F
    d = f[None, None, :] + gx[..., None] * r + gy[..., None] * u
    o = pos - KNOB                                        # the camera from the dial's middle (plate top at y 0)
    best = np.full(gx.shape, np.inf)
    col = np.zeros(gx.shape + (3,))

    def hit(t, mask, c):
        nonlocal best, col
        m = mask & (t > 0) & (t < best)
        best = np.where(m, t, best)
        col = np.where(m[..., None], c, col)

    def cyl(R, y0, y1):
        a = d[..., 0] ** 2 + d[..., 2] ** 2
        b = 2 * (o[0] * d[..., 0] + o[2] * d[..., 2])
        cc = o[0] ** 2 + o[2] ** 2 - R * R
        disc = b * b - 4 * a * cc
        t = (-b - np.sqrt(np.maximum(disc, 0))) / (2 * a)
        y = o[1] + t * d[..., 1]
        return t, (disc > 0) & (y >= y0) & (y <= y1), o + t[..., None] * d

    def disc_at(yp, R):
        t = (yp - o[1]) / d[..., 1]
        p = o + t[..., None] * d
        return t, (p[..., 0] ** 2 + p[..., 2] ** 2) <= R * R, p

    glow = (0.55 + 0.45 * (value / 10)) * (1 + 1.6 * power)
    ink = np.array([255, 225, 160])
    lit_upto = 11 * min(1.0, power * 1.25) - 0.5
    # the light: the water's, from above and a little behind the diver and left; and the dial's glow from its scale
    L = up * 0.85 - fwd * 0.35 - side * 0.4
    L /= np.linalg.norm(L)

    def shade(albedo, nrm, cav, glow_near):
        """Lit: the water's light on the normal, the crevices dark, the dial's warm glow near it."""
        diff = np.clip(nrm @ L, 0, 1)[..., None]
        amb = 0.1 + 0.08 * np.clip(nrm @ up, 0, 1)[..., None]
        grey = albedo.mean(-1, keepdims=True)
        dull = grey + 0.45 * (albedo - grey)                      # murky: the colour mostly gone
        murk = np.array([0.7, 0.9, 1.0])                           # what light there is, the water's blue-green
        warm = np.array([1.0, 0.8, 0.55]) * glow_near[..., None] ** 2 * 0.5 * min(glow, 2.0)
        return dull * (murk * (amb + 0.42 * diff) + warm) * (1 - 0.8 * cav[..., None])

    def tex(mp, key, iv, iu):
        a = mp[key]
        S = a.shape[0] - 1
        return a[np.clip(iv, 0, S), np.clip(iu, 0, S)]

    def fouled_albedo(base, mp, iv, iu):
        cr, ru, ve, ba = (tex(mp, k, iv, iu)[..., None] for k in ("crust", "rust", "verd", "barn"))
        c = base * (1 - 0.75 * ru) + np.array([70, 40, 25]) * 0.75 * ru
        c = c * (1 - 0.6 * ve) + np.array([40, 66, 56]) * 0.6 * ve
        c = c * (1 - 0.85 * cr) + np.array([82, 77, 66]) * (0.75 + 0.5 * tex(mp, "h", iv, iu)[..., None]) * 0.85 * cr
        c = c * (1 - 0.9 * ba) + np.array([118, 112, 100]) * 0.9 * ba
        return c

    def bumped(mp, iv, iu, tangent_u, tangent_v, normal, k=1.0):
        """The surface's normal tipped by its relief (the height's slopes along its two ways)."""
        du, dv = tex(mp, "gx", iv, iu)[..., None] * k, tex(mp, "gz", iv, iu)[..., None] * k
        nn = normal - du * tangent_u - dv * tangent_v
        return nn / np.linalg.norm(nn, axis=-1, keepdims=True)

    def uv(p, scale, size, off=0.0):
        """A surface point's place in a map: across (the side) and along (forward), its middle the dial's."""
        fx = (p[..., 0] * side[0] + p[..., 2] * side[2]) / scale
        fz = (p[..., 0] * fwd[0] + p[..., 2] * fwd[2]) / scale
        return (np.clip(((1 - (fz + 1) / 2) * (size - 1) + off).astype(int), 0, size - 1), np.clip(((fx + 1) / 2 * (size - 1) + off).astype(int), 0, size - 1))

    def near_glow(p):
        return np.exp(-np.maximum(np.hypot(p[..., 0], p[..., 2]) - 3.0, 0) / 2.2) * np.exp(np.minimum(p[..., 1], 0) / 3.0)

    # --- the rock: craggy, its outline broken, ledged and cracked, lit with shadows in its crevices ---
    RS = FACE["rockside"]

    def rock_r(th, y):
        tu = np.clip(((th / (2 * math.pi) + 0.5) * 1023).astype(int), 0, 1023)
        tv = np.clip(((-y) * 14).astype(int) % 1024, 0, 1023)
        crag = RS[tv, tu] - 0.5
        ledge = 0.35 * np.sign(np.sin(y * 1.1 + 2 * np.sin(th * 2))) * (np.abs(np.sin(y * 1.1 + 2 * np.sin(th * 2))) > 0.7)
        return ROCK_R * (1 + 0.12 * np.sin(3 * th + 0.4) + 0.07 * np.sin(7 * th + 0.25 * y)) + 1.6 * crag + ledge - 0.06 * y
    t, m, p = cyl(ROCK_R * 1.4 + 4, -60, -PLATE_T)
    for _ in range(5):
        th = np.arctan2(p[..., 2], p[..., 0])
        R = rock_r(th, p[..., 1])
        a = d[..., 0] ** 2 + d[..., 2] ** 2
        b = 2 * (o[0] * d[..., 0] + o[2] * d[..., 2])
        cc = o[0] ** 2 + o[2] ** 2 - R * R
        disc_ = b * b - 4 * a * cc
        t = (-b - np.sqrt(np.maximum(disc_, 0))) / (2 * a)
        p = o + t[..., None] * d
        m = (disc_ > 0) & (p[..., 1] <= -PLATE_T) & (p[..., 1] >= -60)
    th = np.arctan2(p[..., 2], p[..., 0])
    # its normal from the radius's slopes round and down
    e = 0.02
    R0 = rock_r(th, p[..., 1])
    dRth = (rock_r(th + e, p[..., 1]) - R0) / e
    dRy = (rock_r(th, p[..., 1] + 0.05) - R0) / 0.05
    radial = np.stack([np.cos(th), np.zeros_like(th), np.sin(th)], -1)
    tang = np.stack([-np.sin(th), np.zeros_like(th), np.cos(th)], -1)
    nrm = radial - (dRth / np.maximum(R0, 1))[..., None] * tang - dRy[..., None] * up
    nrm /= np.linalg.norm(nrm, axis=-1, keepdims=True)
    cav = np.clip((0.5 - RS[np.clip(((-p[..., 1]) * 14).astype(int) % 1024, 0, 1023), np.clip(((th / (2 * math.pi) + 0.5) * 1023).astype(int), 0, 1023)]) * 2.5, 0, 1)
    rockc = np.array([72, 62, 52]) * (0.8 + 0.4 * RS[np.clip((p[..., 1] * -41).astype(int) % 1024, 0, 1023), np.clip(((th / (2 * math.pi) + 0.5) * 1023 * 3).astype(int) % 1024, 0, 1023)])[..., None]
    hit(t, m, shade(rockc, nrm, cav, near_glow(p)))
    # its top round the plate: uneven, pitted, weed in the hollows
    t, m, p = disc_at(-PLATE_T, ROCK_R * 1.6)
    rr = np.hypot(p[..., 0], p[..., 2])
    m = m & (rr <= rock_r(np.arctan2(p[..., 2], p[..., 0]), np.full(rr.shape, -PLATE_T)))
    iv, iu = uv(p, ROCK_R * 1.6, 1024)
    mp = FACE["rock"]
    weed = (FACE["weed"][iv, iu] * (rr > PLATE_R + 0.2))[..., None]
    alb = fouled_albedo(np.array([78, 66, 54]), mp, iv, iu)
    alb = alb * (1 - weed) + np.array([30, 52, 24]) * weed
    nrm = bumped(mp, iv, iu, side, fwd, up, 1.6)
    contact = 1 - 0.7 * np.exp(-np.maximum(rr - PLATE_R, 0) / 0.5)[..., None]   # the plate's shadow on the rock
    hit(t, m, shade(alb, nrm, tex(mp, "cav", iv, iu), near_glow(p)) * contact)
    # the dial trembles as it powers up (the rock doesn't)
    o = o + side * shake[0] + fwd * shake[1]
    # the plate: its rim chipped (its radius by its angle), its face worn, dented, scratched, crusted, barnacled
    def plate_r(th):
        tu = np.clip(((th / (2 * math.pi) + 0.5) * 1023).astype(int), 0, 1023)
        return PLATE_R - 0.25 * np.clip((RS[600, tu] - 0.55) * 4, 0, 1)
    t, m, p = cyl(PLATE_R, -PLATE_T, 0)
    m = m & (np.hypot(p[..., 0], p[..., 2]) <= plate_r(np.arctan2(p[..., 2], p[..., 0])) + 0.01)
    iv, iu = uv(p * 0.98, PLATE_R, 1024)
    mp = FACE["plate"]
    nrm = np.stack([p[..., 0], np.zeros_like(p[..., 0]), p[..., 2]], -1) / PLATE_R
    hit(t, m, shade(fouled_albedo(np.array([66, 66, 72]), mp, iv, iu), nrm, tex(mp, "cav", iv, iu), near_glow(p)))
    t, m, p = disc_at(0.0, PLATE_R)
    m = m & (np.hypot(p[..., 0], p[..., 2]) <= plate_r(np.arctan2(p[..., 2], p[..., 0])))
    iv, iu = uv(p, PLATE_R, 1024)
    numb = FACE["num"][iv, iu]
    amount = FACE["ink"][iv, iu] * np.where(numb >= 0, 1 + 1.4 * np.clip(lit_upto - numb, 0, 1), 1.0)
    nrm = bumped(mp, iv, iu, side, fwd, up, 1.3)
    c = shade(fouled_albedo(np.array([46, 47, 52]), mp, iv, iu), nrm, tex(mp, "cav", iv, iu), near_glow(p))
    g = np.clip(amount, 0, 1.6)[..., None]
    stain = (0.55 + 0.45 * tex(mp, "rust", iv, iu))[..., None]
    paint = np.array([150, 140, 115]) * (1 - stain * 0.4) * 0.5 + ink * 0.5 * np.clip(amount - FACE["ink"][iv, iu], 0, 1)[..., None]
    fade = 0.35 + 0.65 * min(1.0, power * 1.25)
    hit(t, m, c * (1 - np.clip(g, 0, 1) * 0.7) + (paint * (1 - fade) + ink * glow * 0.8 * fade) * g * (0.45 + 0.55 * fade))
    # the knob: knurled sides worn smooth in places, a flat top dented and crusted, its pointer
    ang = math.radians(-SWEEP / 2 + SWEEP * value / 10)
    pdir = side * math.sin(ang) + fwd * math.cos(ang)
    o0 = o
    o = o0 - side * wobble
    # its skirt: a flange at the plate with the pointer's line out to the scale
    SK, SKH = 2.45, 0.14
    t, m, p = cyl(SK, 0, SKH)
    nrm = np.stack([p[..., 0], np.zeros_like(p[..., 0]), p[..., 2]], -1) / SK
    iv, iu = uv(p, SK, 512)
    hit(t, m, shade(fouled_albedo(np.array([60, 60, 64]), FACE["skirt"], iv, iu), nrm, tex(FACE["skirt"], "cav", iv, iu), near_glow(p)))
    t, m, p = disc_at(SKH, SK)
    al = p[..., 0] * pdir[0] + p[..., 2] * pdir[2]
    ac = np.abs(p[..., 0] * pdir[2] - p[..., 2] * pdir[0])
    sline = ((al > KNOB_R * 0.8) & (ac < 0.11)).astype(float)
    iv, iu = uv(p, SK, 512)
    mp = FACE["skirt"]
    sline = sline * (1 - tex(mp, "crust", iv, iu) * 0.7)
    sk = shade(fouled_albedo(np.array([44, 44, 48]), mp, iv, iu), bumped(mp, iv, iu, side, fwd, up, 1.2), tex(mp, "cav", iv, iu), near_glow(p))
    hit(t, m, sk * (1 - sline[..., None]) + ink * glow * (0.4 + 0.6 * min(1.0, power * 1.25)) * sline[..., None])
    t, m, p = cyl(KNOB_R, 0, KNOB_H)
    phi = np.arctan2(p[..., 2], p[..., 0])
    mp = FACE["side"]
    sv = np.clip(((phi / (2 * math.pi) + 0.5) * 511).astype(int), 0, 511)
    sh = np.clip(((p[..., 1] / KNOB_H) * 511).astype(int), 0, 511)
    # the knurling, worn down in patches
    worn = np.clip((tex(mp, "h", sh // 4 + 100, sv) + 0.3) * 1.5, 0, 1)
    ridge = np.sin(phi * 36)
    radial = np.stack([np.cos(phi), np.zeros_like(phi), np.sin(phi)], -1)
    tang = np.stack([-np.sin(phi), np.zeros_like(phi), np.cos(phi)], -1)
    nrm = radial + (0.8 * np.cos(phi * 36) * (1 - worn))[..., None] * tang
    nrm /= np.linalg.norm(nrm, axis=-1, keepdims=True)
    notch = (radial @ pdir) > 0.985
    alb = fouled_albedo(np.array([48, 48, 52]) * (0.85 + 0.15 * ridge[..., None]), mp, sh, sv)
    side_col = shade(alb, nrm, tex(mp, "cav", sh, sv) + 0.3 * (1 - worn) * (ridge < -0.6), near_glow(p) * 0.6)
    side_col = np.where(notch[..., None], ink * glow * (0.4 + 0.6 * min(1.0, power * 1.25)), side_col)
    hit(t, m, side_col)
    t, m, p = disc_at(KNOB_H, KNOB_R)
    along = p[..., 0] * pdir[0] + p[..., 2] * pdir[2]
    across = np.abs(p[..., 0] * pdir[2] - p[..., 2] * pdir[0])
    line = (along > 0.1) & (across < 0.13)
    iv, iu = uv(p, KNOB_R, 512)
    mp = FACE["top"]
    top = shade(fouled_albedo(np.array([50, 50, 54]), mp, iv, iu), bumped(mp, iv, iu, side, fwd, up, 1.4), tex(mp, "cav", iv, iu), near_glow(p) * 0.5)
    lw = (1 - tex(mp, "crust", iv, iu) * 0.6)[..., None]
    hit(t, m, np.where(line[..., None], top * (1 - lw) + ink * glow * (0.4 + 0.6 * min(1.0, power * 1.25)) * lw, top))
    o = o0
    # hazed, hidden where the seabed is nearer
    ahead = best * 1.0                                    # rays are f + ...: t is the distance ahead along f
    bed = np.minimum(depth.astype(float), 999).repeat(8 * SS, 0).repeat(8 * SS, 1)
    seen = np.isfinite(best) & (ahead <= bed + 1)
    h = haze(np.where(seen, ahead, 0))
    c = col * (1 - h[..., None]) + far * h[..., None]
    cover = seen.reshape(H, SS, W, SS).mean((1, 3))
    c = (c * seen[..., None]).reshape(H, SS, W, SS, 3).sum((1, 3)) / np.maximum(seen.reshape(H, SS, W, SS).sum((1, 3)), 1)[..., None]
    out = frame * (1 - cover[..., None]) + c * cover[..., None]
    # its glow on the water round it: a soft light, brighter turned up
    cy_, cx_ = np.nonzero(cover > 0)
    if len(cx_):
        rel = KNOB - pos
        a = rel @ f
        sx, sy = W / 2 + F * (rel @ r) / a, H / 2 - F * (rel @ u) / a
        rad = F * PLATE_R * 1.6 / a
        yy, xx = np.mgrid[0:H, 0:W]
        g = np.exp(-((xx - sx) ** 2 + (yy - sy) ** 2 * 2.2) / (2 * rad * rad * (1 + power))) * 0.10 * glow
        out = out + (np.array([255, 210, 140]) - out) * (g * (1 - cover))[..., None]
    return out


# the sticky turn, frame by frame from STOP1: the dial's value and the knob's shudder (units across); two small
# jerky starts that catch and stick, a shudder, then it gives and snaps over with one click to 10
TURN = {308: (9.0, 0), 309: (9.0, 0), 310: (9.0, 0), 311: (9.0, 0), 312: (9.0, 0),
        313: (9.07, 0.05), 314: (9.11, -0.04), 315: (9.04, 0.03), 316: (9.06, -0.03), 317: (9.04, 0.02), 318: (9.05, -0.01),
        319: (9.04, 0), 320: (9.04, 0), 321: (9.04, 0), 322: (9.14, 0.06), 323: (9.18, -0.05), 324: (9.10, 0.04),
        325: (9.12, -0.03), 326: (9.10, 0.02), 327: (9.11, -0.02), 328: (9.10, 0.01), 329: (9.10, 0), 330: (9.62, 0),
        331: (10.0, 0), 332: (10.0, 0)}
CLICK = 331
PUFFS = [314, 323]


def screen(cam, p):
    pos, f, r, u, _ = cam
    rel = p - pos
    a = rel @ f
    return W / 2 + F * (rel @ r) / a, H / 2 - F * (rel @ u) / a, a


def silt(img, cam, n):
    """A puff of silt off the rock round the plate as the knob sticks: soft grey-brown blobs rising and spreading,
    fading over a second."""
    from PIL import ImageFilter
    lay = np.zeros((H, W))
    rng = np.random.default_rng(7)
    for k0 in PUFFS:
        age = n - k0
        if not (0 <= age < 14):
            continue
        r2 = np.random.default_rng(k0)
        for i in range(18):
            a = r2.uniform(0, 2 * math.pi)
            rad = PLATE_R + 0.3 + r2.uniform(0, 1.2) + 0.12 * age
            p = KNOB + np.array([math.cos(a) * rad, -PLATE_T + 0.1 + 0.08 * age * r2.uniform(0.5, 1.5), math.sin(a) * rad])
            x, y, dist = screen(cam, p)
            size = (0.35 + 0.05 * age) * F / dist
            box = (slice(max(int(y - 3 * size), 0), min(int(y + 3 * size) + 1, H)), slice(max(int(x - 3 * size), 0), min(int(x + 3 * size) + 1, W)))
            if box[0].start >= box[0].stop or box[1].start >= box[1].stop:
                continue
            gy, gx = np.mgrid[box[0], box[1]]
            lay[box] += 0.5 * (1 - age / 14) * np.exp(-((gx - x) ** 2 + (gy - y) ** 2) / (2 * size * size))
    _ = rng
    a = np.clip(lay, 0, 0.55)[..., None]
    return img * (1 - a) + np.array([92, 86, 72]) * a


def sparks(img, cam, n):
    """The click: a crackle of electric arcs off the dial's rim and a white-blue flash filling the frame."""
    from PIL import ImageFilter
    rng = np.random.default_rng(n * 31 + 5)
    arcs = Image.new("L", (W, H), 0)
    dr = ImageDraw.Draw(arcs)
    count = 9 if n == CLICK else 14
    for i in range(count):
        a = rng.uniform(0, 2 * math.pi)
        p = KNOB + np.array([math.cos(a) * PLATE_R, 0.05, math.sin(a) * PLATE_R])
        x, y, dist = screen(cam, p)
        pts = [(x, y)]
        ang = math.atan2(y - H / 2 * 0.9, x - W / 2) + rng.uniform(-0.6, 0.6)
        length = rng.uniform(120, 420) * (1.4 if n > CLICK else 1)
        steps = 9
        for k in range(steps):
            ang += rng.uniform(-0.7, 0.7)
            x += math.cos(ang) * length / steps
            y += math.sin(ang) * length / steps - 6
            pts.append((x, y))
        dr.line(pts, fill=255, width=3)
        # a fork
        j = rng.integers(2, steps)
        fx, fy = pts[j]
        fork = [(fx, fy)]
        fa = ang + rng.uniform(-1.2, 1.2)
        for k in range(4):
            fa += rng.uniform(-0.6, 0.6)
            fx += math.cos(fa) * length / 12
            fy += math.sin(fa) * length / 12
            fork.append((fx, fy))
        dr.line(fork, fill=200, width=2)
    core = np.asarray(arcs).astype(float) / 255
    glow = np.asarray(arcs.filter(ImageFilter.GaussianBlur(9))).astype(float) / 255 * 3
    blue = np.array([170, 210, 255])
    out = img + (np.array([255, 255, 255]) - img) * np.clip(core, 0, 1)[..., None]
    out = out + (blue - out) * np.clip(glow, 0, 1)[..., None]
    flash = 0.45 if n == CLICK else 0.9
    return out + (np.array([225, 240, 255]) - out) * flash


ORCA0, ORCA1 = 282, 328             # the larger orca back across, right to left, behind the dial (the approach, the first catch)
ORCA = None


def orca_layer(img, depth, cam, n):
    """The larger of the pair (the same model, length, swim and haze as at 84..156), far behind the dial: a slow
    crossing right to left, 55 beyond the knob and below it, seen over the dial in the haze, its heading along its path."""
    global ORCA
    if not ORCA0 <= n <= ORCA1:
        return img
    import orca
    if ORCA is None:
        ORCA = orca.load_model()
    V, Fc = ORCA
    pos, f, r, u, _ = cam
    fwd = np.array([f[0], 0.0, f[2]]); fwd /= np.linalg.norm(fwd)
    side = np.cross(np.array([0.0, 1.0, 0.0]), fwd)
    t = (n - ORCA0) / (ORCA1 - ORCA0)
    head = -side + fwd * 0.12
    head /= np.linalg.norm(head)
    p = KNOB + fwd * 55.0 + np.array([0.0, -21.0 + 2.0 * math.sin(math.pi * t), 0.0]) + side * (70.0 - 140.0 * t)
    z = V[:, 2]
    tail = np.clip(0.55 - z, 0, None)
    B = V.copy()
    B[:, 0] += 0.035 * tail ** 1.6 * np.sin(2 * math.pi * (1.1 * z - 0.32 * n / 12 * 2))
    up = np.array([0.0, 1.0, 0.0])
    x = np.cross(up, head); x /= np.linalg.norm(x)
    y = np.cross(head, x)
    P = p + 46.0 * (B[:, :1] * x + B[:, 1:2] * y + B[:, 2:3] * head)
    return orca.render(img, depth, cam, P, Fc, against=True)


def water_light(img, depth, n):
    """The dim light in the open water beyond the dial (where the dive's depth is open), faint, blue-green, from
    above; coming up as the camera settles, so the orca crossing it reads as a dark shape."""
    k = min(max((n - 262) / 30, 0.0), 1.0)
    if k <= 0:
        return img
    k = k * k * (3 - 2 * k)
    open_ = (depth >= 999).astype(float).repeat(8, 0).repeat(8, 1)
    yy = np.arange(H)[:, None] / H
    lift = np.exp(-yy / 0.35)[..., None] * np.array([12.0, 22.0, 26.0]) * k
    return img + lift * open_[..., None]


def finish(img, depth, cam, n):
    """A frame of the ending: the knob (its value, its shudder, how far powered up), the silt, the debris, the
    sparks and the lightning, the flash."""
    img = water_light(img, depth, n)
    img = orca_layer(img, depth, cam, n)
    img = knob_layer(img, depth, cam, value_at(n), n, shudder(n), power_at(n), shake_at(n))
    img = silt(img, cam, n)
    img = debris(img, cam, n)
    if n == CLICK:
        img = crackle(img, cam, n, 3, 90, 2, 0.12)
    elif CLICK < n < BOLT:
        # small sparks, more as it powers up
        p = power_at(n)
        img = crackle(img, cam, n, int(1 + 5 * p), 60 + 120 * p, 2, 0.04 * p)
    elif n >= BOLT and n < LAST:
        img = crackle(img, cam, n, 16, 700, 4, 0.35 if n == BOLT else 0.6)
    elif n == LAST:
        img = crackle(img, cam, n, 18, 900, 5, 0.95)
    return img


def value_at(n):
    return 10.0 if n >= CLICK else TURN.get(n, (9.0, 0))[0]


def power_at(n):
    return min(max((n - CLICK) / (POWER1 - CLICK + 1), 0.0), 1.0)


def shake_at(n):
    """The dial's tremble as it powers up: a hum, rising (units across and along)."""
    p = power_at(n)
    if p == 0:
        return (0.0, 0.0)
    rng = np.random.default_rng(n)
    a = 0.012 + 0.05 * p
    return (rng.uniform(-a, a), rng.uniform(-a, a) * 0.6)


def debris(img, cam, n):
    """Silt and bits of crust shaken loose as it powers up: small flakes leaving the plate's rim, drifting out and
    up, fading."""
    if n <= CLICK:
        return img
    out = img.copy()
    rng = np.random.default_rng(77)
    for i in range(140):
        born = CLICK + 1 + rng.integers(0, POWER1 - CLICK)
        a = rng.uniform(0, 2 * math.pi)
        r0 = rng.uniform(1.8, PLATE_R + 0.2)
        speed = rng.uniform(0.05, 0.22)
        rise = rng.uniform(0.02, 0.12)
        pale = rng.random() < 0.45
        size = rng.uniform(0.025, 0.07)
        age = n - born
        if age < 0 or age > 16:
            continue
        p = KNOB + np.array([math.cos(a) * (r0 + speed * age), 0.1 + rise * age, math.sin(a) * (r0 + speed * age)])
        x, y, dist = screen(cam, p)
        px = max(1.0, size * F / dist)
        x0, x1, y0, y1 = int(x - px), int(x + px) + 1, int(y - px), int(y + px) + 1
        if x1 <= 0 or y1 <= 0 or x0 >= W or y0 >= H:
            continue
        x0, y0, x1, y1 = max(x0, 0), max(y0, 0), min(x1, W), min(y1, H)
        col = np.array([110, 104, 92]) if pale else np.array([58, 52, 40])
        k = 0.85 * (1 - age / 17)
        out[y0:y1, x0:x1] = out[y0:y1, x0:x1] * (1 - k) + col * k
    return out


def crackle(img, cam, n, count, reach, width, flash):
    """Electric arcs off the dial's rim: jagged, forking, white with a blue glow; and the frame flashed toward
    white-blue."""
    from PIL import ImageFilter
    rng = np.random.default_rng(n * 31 + 5)
    arcs = Image.new("L", (W, H), 0)
    dr = ImageDraw.Draw(arcs)
    for i in range(count):
        a = rng.uniform(0, 2 * math.pi)
        p = KNOB + np.array([math.cos(a) * PLATE_R * rng.uniform(0.6, 1.0), 0.1, math.sin(a) * PLATE_R * rng.uniform(0.6, 1.0)])
        x, y, dist = screen(cam, p)
        pts = [(x, y)]
        cx, cy, _ = screen(cam, KNOB)
        ang = math.atan2(y - cy, x - cx) + rng.uniform(-0.5, 0.5)
        length = rng.uniform(0.5, 1.0) * reach
        steps = 10
        for k in range(steps):
            ang += rng.uniform(-0.7, 0.7)
            x += math.cos(ang) * length / steps
            y += math.sin(ang) * length / steps
            pts.append((x, y))
            if rng.random() < 0.3:
                fx, fy, fa = x, y, ang + rng.uniform(-1.3, 1.3)
                fork = [(fx, fy)]
                for _ in range(4):
                    fa += rng.uniform(-0.6, 0.6)
                    fx += math.cos(fa) * length / 14
                    fy += math.sin(fa) * length / 14
                    fork.append((fx, fy))
                dr.line(fork, fill=190, width=max(1, width - 1))
        dr.line(pts, fill=255, width=width)
    core = np.asarray(arcs).astype(float) / 255
    glow = np.asarray(arcs.filter(ImageFilter.GaussianBlur(4 + 2 * width))).astype(float) / 255 * 3
    out = img + (np.array([255, 255, 255]) - img) * np.clip(core, 0, 1)[..., None]
    out = out + (np.array([170, 210, 255]) - out) * np.clip(glow, 0, 1)[..., None]
    return out + (np.array([225, 240, 255]) - out) * flash


def shudder(n):
    return TURN.get(n, (9.0, 0))[1]


def main():
    if sys.argv[1] == "dive":
        dive()
        return
    out = sys.argv[1]
    small = "--small" in sys.argv
    cams = cameras(WORK + "/script.txt")
    video = np.memmap(WORK + "/video_hd.rgb", np.uint8, "r").reshape(-1, H, W, 3)
    depth = np.fromfile(WORK + "/depth.bin", "<i2").reshape(-1, 96, 192)
    train = np.memmap(TRAIN, np.uint8, "r").reshape(-1, H, W, 3)
    size = "768x384" if small else f"{W}x{H}"
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", "12",
                           "-i", "-", "-vf", f"scale={size}:flags=neighbor", "-c:v", "libx264", "-crf", "18", "-profile:v", "main",
                           "-level", "4.0", "-bf", "0", "-g", "12", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out],
                          stdin=subprocess.PIPE)
    for n in range(228, LAST + 1):
        img = np.array(video[n]).astype(float)
        # the knob from where it first shows in the dark ahead
        img = finish(img, depth[n], cams[n], n)
        ff.stdin.write(np.clip(np.round(img), 0, 255).astype(np.uint8).tobytes())
    # the hard cut: the train's first frames
    for k in range(36):
        ff.stdin.write(np.array(train[k]).tobytes())
    ff.stdin.close()
    ff.wait()


if __name__ == "__main__":
    main()
