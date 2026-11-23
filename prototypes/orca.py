#!/usr/bin/env python3
"""Design prototype: something enormous passes through the dive, seen only as a silhouette.

The model is the Killer Whale by Poly by Google (CC-BY, poly.pizza/m/7pqZEQ9b_E-, 636 triangles), scaled up to a
leviathan. It is never lit: each pixel it covers is a dark body colour mixed toward the water's far colour by the
haze between it and the camera (the dive's own fog curve), so its size reads from what it blocks: the lit water,
the canyon's haze, the diver's lamp on the floor. The dive's seabed and kelp hide it where they are nearer (the
dive's 192x96 depth buffer). It swims: a wave down its body, growing to the flukes (for the build: a few swim
phases of the mesh, as the fish atlas has).

Over the HD dive (segments/deep/video_hd.rgb) with the dive's own camera (deep/data/script.txt).

    python orca.py VARIANT OUT.mp4 [--small]

VARIANT 6: as v1 (the dive's camera as it is, no shadow) but far out and low in the canyon's haze, a calm crossing
over 6 s keeping its distance, a gentler swim, and a second, smaller orca beside and behind it, its swim out of step;
7: the same, the leader alone.
VARIANT 4 (v3 slower, to the end of its exit, the camera looking up at it overhead): the dive's frames are
rendered again for the tilted camera by the dive's own reference (run first: python orca.py tilt 4), into WORK.
VARIANT 5 (v4 kept high near the surface the whole pass, the camera looking up steeply to find it whole against the
surface light, following it as it swims on ahead near the surface until it fades; its shadow larger and softer):
run python orca.py tilt 5 first (into WORK + "5").

VARIANT 1: across the far haze, out over the canyon (frames 108..152); 2: close in front of the diver, below the
eye, across the lamp's pool, never seen whole (frames 112..150); 3: from behind and above, its shadow sweeping
across the floor first (the light from the surface a little behind the diver), then overtaking the diver into
view from the top of the frame and on ahead and down into the haze (frames 100..164).
Needs numpy, Pillow, trimesh; the model at $CARD_ATTACK_WORK/models/creatures/7pqZEQ9b_E-.glb and the
dive's depth dump at $CARD_ATTACK_WORK/dive-hd-preview/depth.bin (from the dive's look preview, which is not in
this repository).
"""
import math
import os
import subprocess
import sys

import numpy as np
import trimesh
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))

def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


DEEP = os.path.join(HERE, "..", "segments", "deep") + "/"
MODEL = _work("models", "creatures", "7pqZEQ9b_E-.glb")
DEPTH = _work("dive-hd-preview", "depth.bin")
SCRIPT = os.path.join(HERE, "..", "segments", "deep", "data", "script.txt")
W, H, F = 1536, 768, 800
SS = 3
WORK = _work("orca")
DARK = np.array([2.0, 4.0, 7.0])


def load_model():
    m = trimesh.load(MODEL, force="scene").to_geometry()
    V = np.asarray(m.vertices, float)
    V = V - (V.max(0) + V.min(0)) / 2
    return V / (V[:, 2].max() - V[:, 2].min()), np.asarray(m.faces)   # length 1 along z, the head at +z


def cameras(script=SCRIPT):
    cams = []
    for line in open(script):
        v = list(map(int, line.split()))
        pos = np.array([v[1] / 100, v[2] + v[3] / 100, v[4] + v[5] / 10])
        f, r, u = (np.array(v[24:27]) / 100, np.array(v[27:30]) / 100, np.array(v[30:33]) / 100)
        light = v[15] / 32
        far = np.array([1, 3, 8]) + (np.array([6, 40, 62]) - np.array([1, 3, 8])) * light
        cams.append((pos, f, r, u, far))
    return cams


def haze(d, reach=70.0):
    """The dive's fog: how much of the water's far colour lies between the camera and a thing d units ahead (reach
    70). Looking up into the lit water at a dark shape (v5) its outline carries further: reach 120."""
    return np.clip(0.85 * (np.maximum(d, 0) / reach) ** 0.8, 0, 1)


def pose(V, n, variant, cams, which=0):
    """The orca in the world at frame n: its length, place, heading, and the swim wave down its body. (v6: which
    of the pair, 0 the leader, 1 the smaller one beside and behind it.)"""
    wave, beat = 0.06, 0.55
    if variant in (6, 7):
        # v6: far out and low in the canyon's haze, a calm crossing left to right over 6 s; it keeps its distance
        # from the diver (its place moves on with the diver's), its heading along its path
        t = (n - 84) / 72
        pos, f, r, u, _ = cams[n]
        base = np.array([-38.0 + 70.0 * t, pos[1] - 19.0, pos[2] + 66.0])
        vel = np.array([70.0 / 72, 0.0, 1.1])
        head = vel / np.linalg.norm(vel)
        side = np.cross(np.array([0.0, 1.0, 0.0]), head)
        L = 46.0
        wave, beat = 0.035, 0.32
        if which == 1:
            # the second: smaller, behind and a little further out and lower, drifting nearer and back again
            L = 37.0
            base = base - head * (14.0 + 5.0 * math.sin(2 * math.pi * t * 0.8)) - side * (7.0 + 4.0 * math.sin(2 * math.pi * t * 0.6 + 1.0)) \
                + np.array([0.0, -3.0 + 1.5 * math.sin(2 * math.pi * t * 0.5), 0.0])
        p = base
    elif variant == 1:
        # out over the canyon, across the far haze, left to right, a little below the diver
        t = (n - 108) / 44
        L = 48.0
        start, end = np.array([-70.0, -26.0, 232.0]), np.array([34.0, -20.0, 246.0])
        p = start + (end - start) * t
        head = (end - start) / np.linalg.norm(end - start)
    elif variant == 3:
        # from behind and above the diver, overtaking it and diving on ahead into the haze: a smooth path through
        # four places (world units), its heading along the path
        p, head = path3(n, KEYS3)
        L = 52.0
    elif variant == 4:
        p, head = path3(n, KEYS4)
        L = 52.0
    elif variant == 5:
        p, head = path3(n, KEYS5)
        L = 52.0
    else:
        # close in front of the diver, below the eye, across the lamp's pool, right to left; it keeps pace with the
        # diver as it crosses (its place from the diver's at the window's middle: 20 ahead, 36 right to 36 left)
        t = (n - 112) / 38
        L = 40.0
        pos, f, r, u, _ = cams[n]
        _, f0, r0, u0, _ = cams[131]
        p = pos + 20 * f0 + (36 - 72 * t) * r0 + (-3 + 4 * t) * u0
        head = -r0
    # the swim: a wave running back down the body, growing toward the flukes (sideways in the body's frame)
    z = V[:, 2]
    tail = np.clip(0.55 - z, 0, None)
    sway = wave * tail ** 1.6 * np.sin(2 * math.pi * (1.1 * z - beat * n / 12 * 2) + (math.pi * 0.8 if which == 1 else 0.0))
    B = V.copy()
    B[:, 0] += sway
    # body frame: z along the heading, y up (kept level), x across
    up = np.array([0.0, 1.0, 0.0])
    x = np.cross(up, head); x /= np.linalg.norm(x)
    y = np.cross(head, x)
    return p + L * (B[:, :1] * x + B[:, 1:2] * y + B[:, 2:3] * head)


KEYS3 = [(100, (6.0, 32.0, 85.0)), (112, (4.0, 30.0, 115.0)), (126, (1.0, 26.0, 142.0)), (140, (-2.0, 8.0, 170.0)), (152, (-8.0, -14.0, 198.0)), (166, (-15.0, -34.0, 228.0))]
WINDOW = {1: (108, 152), 2: (112, 150), 3: (104, 162), 4: (100, 222), 5: (92, 178), 6: (84, 156), 7: (84, 156)}
# v5: as v4's speed (1.86 units a frame), but high near the surface (y about 46) the whole pass: its shadow on the
# floor 54 units ahead of it (the light's slant), overhead, then on ahead near the surface into the haze
# v5: far up near the surface (y 100, about 105 above the diver), so it is smallish overhead (forced perspective by
# distance, the model its own size); 2.6 units a frame (1.5 a frame on the diver: far up, it still crosses the sky
# slowly); the light steeper (17 degrees from straight down) so its shadow, larger and softer, lands just ahead
KEYS5 = [(n, (2.0 + 0.15 * (n - 92), 100.0 + 1.5 * math.sin((n - 92) / 14), 131.0 + 2.6 * (n - 108))) for n in range(78, 220, 14)]
LIGHT5 = np.array([0.0, -1.0, 0.3]) / np.linalg.norm([0.0, -1.0, 0.3])
# v4: slower, overtaking later, overhead while the camera looks up, then down and away until it is gone in the haze
KEYS4 = [(96, (-30.0, 32.0, 70.0)), (112, (-28.0, 30.0, 100.0)), (130, (-24.0, 27.0, 132.0)), (146, (-14.0, 25.0, 164.0)),
         (156, (-4.0, 19.0, 184.0)), (166, (4.0, 4.0, 202.0)), (176, (6.0, -18.0, 222.0)), (190, (2.0, -42.0, 248.0)),
         (204, (-4.0, -62.0, 273.0)), (218, (-10.0, -80.0, 297.0)), (232, (-16.0, -98.0, 321.0))]


def smooth(e0, e1, x):
    t = min(max((x - e0) / (e1 - e0), 0.0), 1.0)
    return t * t * (3 - 2 * t)


def work(variant):
    return WORK if variant == 4 else WORK + str(variant)


def tilt(n, variant=4):
    if variant == 5:
        # up steeply over a second as its shadow goes by (85 degrees, nearly straight up), held while it passes
        # overhead, eased down to 55 degrees to follow it on ahead, held till it fades, then back down
        return math.radians(85 * smooth(110, 124, n) - 10 * smooth(134, 156, n) - 75 * smooth(170, 188, n))
    """v4: how far the camera looks up (radians): up over a second as the shadow has gone by, held while the orca
    passes overhead, then back down to follow it."""
    return math.radians(65) * (smooth(138, 150, n) - smooth(158, 172, n))


def tilted_script(out, variant=4):
    """The dive's script with the camera tilted up about its right vector (v4); everything else as it is."""
    lines = []
    for n, line in enumerate(open(SCRIPT)):
        v = line.split()
        a = tilt(n, variant)
        if a != 0 and v[0] == "0":
            w = list(map(int, v))
            # never past 58 degrees up (the dive's rays must keep going forward, to the top of the screen)
            a = min(a, math.radians(58) - math.asin(max(-1.0, min(1.0, w[7] / 300))))
            for at, k in ((6, 300), (24, 100)):
                f, u = np.array(w[at:at + 3], float) / k, np.array(w[at + 6:at + 9], float) / k
                f2, u2 = f * math.cos(a) + u * math.sin(a), u * math.cos(a) - f * math.sin(a)
                w[at:at + 3] = [round(c * k) for c in f2]
                w[at + 6:at + 9] = [round(c * k) for c in u2]
            v = list(map(str, w))
        lines.append(" ".join(v))
    open(out, "w").write("\n".join(lines) + "\n")


def tilted_dive(variant=4):
    """Render the dive again (its reference, natively) for the tilted camera, with its depths: WORK/video_hd.rgb,
    WORK/depth.bin (192x96 i16 a frame, the depths the creatures are tested against), WORK/script.txt."""
    WORK = work(variant)
    os.makedirs(WORK, exist_ok=True)
    tilted_script(WORK + "/script.txt", variant)
    src = open(os.path.join(HERE, "..", "segments", "deep", "reference.rs")).read()
    deep = os.path.abspath(os.path.join(HERE, "..", "segments", "deep"))
    src = src.replace('#[path = "../../demokit/src/lib.rs"]', '#[path = "%s/../../demokit/src/lib.rs"]' % deep)
    src = src.replace('#[path = "reference_hd.rs"]', '#[path = "%s/reference_hd.rs"]' % deep)
    src = src.replace("        hd::frame(&scene, &hd_data, &trig, &mut hd_rgb);",
                      "        hd::frame(&scene, &hd_data, &trig, &mut hd_rgb);\n        { use std::io::Write as _; let mut df = std::fs::OpenOptions::new().append(true).create(true).open(\"%s/depth.bin\").unwrap(); df.write_all(&under_depth.iter().flat_map(|&d| (d as i16).to_le_bytes()).collect::<Vec<u8>>()).unwrap(); }" % WORK)
    open(WORK + "/reference_tilt.rs", "w").write(src)
    if os.path.exists(WORK + "/depth.bin"):
        os.remove(WORK + "/depth.bin")
    subprocess.run(["rustc", "-O", "-o", WORK + "/reference_tilt", WORK + "/reference_tilt.rs"], check=True)
    data = os.path.join(deep, "data")
    swarm = os.path.abspath(os.path.join(deep, "..", "swarm", "data", "tables.txt"))
    with open(WORK + "/both.rgb", "wb") as fh:
        subprocess.run([WORK + "/reference_tilt", data + "/map.txt", WORK + "/script.txt", data + "/hd.txt", swarm], stdout=fh, check=True)
    lo = 309 * 192 * 96 * 3
    with open(WORK + "/both.rgb", "rb") as fh, open(WORK + "/video_hd.rgb", "wb") as out:
        fh.seek(lo)
        while True:
            b = fh.read(1 << 24)
            if not b:
                break
            out.write(b)
    os.remove(WORK + "/both.rgb")
# the light from the surface, a little from behind the diver (down and forward): its shadow lands ahead of it
LIGHT = np.array([0.0, -1.0, 0.85]) / np.linalg.norm([0.0, -1.0, 0.85])


def path3(n, keys):
    """Catmull-Rom through the keys: the place at frame n and the heading."""
    ks = [k for k, _ in keys]
    pts = [np.array(p) for _, p in keys]
    pts = [2 * pts[0] - pts[1]] + pts + [2 * pts[-1] - pts[-2]]
    i = max(0, min(len(ks) - 2, sum(1 for k in ks if k <= n) - 1))
    t = (n - ks[i]) / (ks[i + 1] - ks[i])
    p0, p1, p2, p3 = pts[i], pts[i + 1], pts[i + 2], pts[i + 3]
    at = lambda t: 0.5 * (2 * p1 + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t * t + (-p0 + 3 * p1 - 3 * p2 + p3) * t ** 3)
    d = at(t + 0.01) - at(t - 0.01)
    return at(t), d / np.linalg.norm(d)


def shadow(frame, depth, cam, P, Fc, soft=0.9, light=None, dark=0.8):
    """The orca's shadow on the dive's seabed and kelp: each pixel's place in the world (its ray to the dive's
    depth there, between the 192x96 samples), taken up along the light to a level plane, where the orca is drawn
    along the light too; soft-edged; the floor darkened under it."""
    pos, f, r, u, _ = cam
    d = depth.astype(float)
    solid = d < 900
    # the depths between the samples (only from solid neighbours), pixel middles
    dd = np.where(solid, d, np.nan)
    ys = (np.arange(H) + 0.5) / 8 - 0.5
    xs = (np.arange(W) + 0.5) / 8 - 0.5
    y0 = np.clip(np.floor(ys).astype(int), 0, 94); x0 = np.clip(np.floor(xs).astype(int), 0, 190)
    fy = np.clip(ys - y0, 0, 1)[:, None]; fx = np.clip(xs - x0, 0, 1)[None, :]
    a = dd[y0][:, x0]; b = dd[y0][:, x0 + 1]; c = dd[y0 + 1][:, x0]; e = dd[y0 + 1][:, x0 + 1]
    near = d.repeat(8, 0).repeat(8, 1)
    lerp = (a * (1 - fx) + b * fx) * (1 - fy) + (c * (1 - fx) + e * fx) * fy
    D = np.where(np.isnan(lerp), near, lerp)
    ok = near < 900
    gx = (np.arange(W) + 0.5 - W / 2) / F
    gy = -(np.arange(H) + 0.5 - H / 2) / F
    ray = f[None, None, :] + gx[None, :, None] * r[None, None, :] + gy[:, None, None] * u[None, None, :]
    Q = pos + ray * D[..., None]
    # up the light to the plane y = 0 (world), and the orca the same way
    plane = 0.0
    L = LIGHT if light is None else light
    up = lambda X: X[..., [0, 2]] + L[[0, 2]] * ((plane - X[..., 1:2]) / L[1])
    q = up(Q)
    o = up(P)
    lo, hi = o.min(0) - 6, o.max(0) + 6
    res = 6.0                                           # raster cells a unit
    nx, nz = int((hi[0] - lo[0]) * res) + 1, int((hi[1] - lo[1]) * res) + 1
    img = Image.new("L", (nx, nz), 0)
    dr = ImageDraw.Draw(img)
    for tri in Fc:
        dr.polygon([((o[i, 0] - lo[0]) * res, (o[i, 1] - lo[1]) * res) for i in tri], fill=255)
    from PIL import ImageFilter
    img = img.filter(ImageFilter.GaussianBlur(radius=soft * res))     # the penumbra: soft, a unit or more
    sh = np.asarray(img).astype(float) / 255
    ix = ((q[..., 0] - lo[0]) * res).astype(int); iz = ((q[..., 1] - lo[1]) * res).astype(int)
    inside = ok & (ix >= 0) & (iz >= 0) & (ix < nx) & (iz < nz)
    s = np.zeros((H, W))
    s[inside] = sh[iz[inside], ix[inside]]
    # only under it (the floor below the orca's middle), and fading out with the haze to the floor
    s *= (Q[..., 1] < P[:, 1].mean()) * (1 - 0.8 * haze(D))
    return frame * (1 - dark * s[..., None])


def render(frame, depth, cam, P, Fc, against=False):
    reach = 120.0 if against else 70.0
    pos, f, r, u, far = cam
    rel = P - pos
    a = np.einsum('ij,j->i', rel, f)
    if (a > 0.5).sum() < 3:
        return frame
    sx = W / 2 + F * np.einsum('ij,j->i', rel, r) / np.maximum(a, 0.5)
    sy = H / 2 - F * np.einsum('ij,j->i', rel, u) / np.maximum(a, 0.5)
    # depth, far to near: each triangle at its middle's depth (painted over what is further)
    da = a[Fc].mean(1)
    order = np.argsort(-da)
    zimg = Image.new("F", (W * SS, H * SS), 0.0)
    dr = ImageDraw.Draw(zimg)
    for k in order:
        if (a[Fc[k]] < 0.5).any():
            continue
        dr.polygon([(sx[i] * SS, sy[i] * SS) for i in Fc[k]], fill=float(da[k]))
    z = np.asarray(zimg)
    # hidden where the dive's seabed or kelp is nearer (its depth buffer, whole units ahead, 8 x 8 a sample)
    bed = np.minimum(depth.astype(float), 999).repeat(8 * SS, 0).repeat(8 * SS, 1)
    seen = (z > 0) & (z <= bed + 1)
    cover = seen.reshape(H, SS, W, SS).mean((1, 3))
    h = np.where(seen, haze(z, reach), 0).reshape(H, SS, W, SS).sum((1, 3)) / np.maximum(seen.reshape(H, SS, W, SS).sum((1, 3)), 1)
    # the haze toward the water's far colour; or (looking up at the surface light) toward the water behind it
    body = DARK * (1 - h[..., None]) + (frame if against else far) * h[..., None]
    out = frame * (1 - cover[..., None]) + body * cover[..., None]
    return out


def main():
    if sys.argv[1] == "tilt":
        tilted_dive(int(sys.argv[2]) if len(sys.argv) > 2 else 4)
        return
    variant = int(sys.argv[1])
    out = sys.argv[2]
    small = "--small" in sys.argv
    V, Fc = load_model()
    if variant in (4, 5):
        cams = cameras(work(variant) + "/script.txt")
        video = np.memmap(work(variant) + "/video_hd.rgb", np.uint8, "r").reshape(-1, H, W, 3)
        depth = np.fromfile(work(variant) + "/depth.bin", "<i2").reshape(-1, 96, 192)
    else:
        cams = cameras()
        video = np.memmap(DEEP + "video_hd.rgb", np.uint8, "r").reshape(-1, H, W, 3)
        depth = np.fromfile(DEPTH, "<i2").reshape(-1, 96, 192)
    first, last = WINDOW[variant]
    frames = range(first - 14, last + 14)
    size = "768x384" if small else f"{W}x{H}"
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", "12",
                           "-i", "-", "-vf", f"scale={size}:flags=neighbor", "-c:v", "libx264", "-crf", "18", "-profile:v", "main",
                           "-level", "4.0", "-bf", "0", "-g", "12", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out],
                          stdin=subprocess.PIPE)
    for n in frames:
        img = np.array(video[n]).astype(float)
        if first <= n <= last:
            if variant == 6:
                # the pair, the further first
                Ps = [pose(V, n, variant, cams, k) for k in (0, 1)]
                Ps.sort(key=lambda P: -float((P.mean(0) - cams[n][0]) @ cams[n][1]))
                for P in Ps:
                    img = render(img, depth[n], cams[n], P, Fc)
                ff.stdin.write(np.clip(np.round(img), 0, 255).astype(np.uint8).tobytes())
                continue
            P = pose(V, n, variant, cams)
            if variant in (3, 4, 5):
                img = shadow(img, depth[n], cams[n], P, Fc, *((2.5, LIGHT5, 0.85) if variant == 5 else (0.9, None, 0.8)))
            img = render(img, depth[n], cams[n], P, Fc, variant == 5)
        ff.stdin.write(np.clip(np.round(img), 0, 255).astype(np.uint8).tobytes())
    ff.stdin.close()
    ff.wait()


if __name__ == "__main__":
    main()
