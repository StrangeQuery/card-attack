"""The night arrival's data (the train's ending), from its prototype (prototypes/train_night.py, which builds on
prototypes/train.py; the models of prototypes/models/build_models.py and build_night_models.py in
$CARD_ATTACK_WORK/models; the stadium's geometry and lights from $CARD_ATTACK_WORK/audience/stadium;
the demos' footage from $CARD_ATTACK_WORK/demos; the source files are not in this repository):

    python segments/night/data/make_data.py        # writes scene.bin and clips.bin here (not kept in git)

scene.bin, in the train's format (segments/train/data/make_data.py: named arrays of 32-bit whole numbers):
  - the world: the ride's world (the old: the frames the night dissolves in from) and the night's (the new: the
    station, the town, the screens, the stadium, the farmland dropped past the town's edge), as one list of
    triangles each flagged old and new, with what each mode selects it by (small differs);
  - what moves: our own train (seen from outside once we're through the windscreen), the drone, the airship (its
    model at its scale, 1.4);
  - the materials (the night's kinds renumbered from 60, but the platform, which is the train's);
  - the lamps, the screens, the board, the stadium's lights, the airship's lights, the up-lights, the balloons, the
    fireworks' shells, the photo flashes, the stars, the flight's keyframes, the overhead line's wires;
  - pictures (assets): the board's dots, the banners, the captions, the stadium's marquee, the airship's words, the
    volume knob's printed parts.
clips.bin: the footage the screens show (the clips; the parodies' stand-ins are worked out on CARDIAC), 160 x 120 x 60 frames each, bytes (an index of which demos first).

Whole numbers: places in centimetres (the film's bent frame, as the train's), normals in hundredths, albedos and
colours in thousandths.
"""
import importlib.util
import json
import math
import os
import struct
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..", "..")
sys.path.insert(0, os.path.join(ROOT, "prototypes"))
sys.path.insert(0, os.path.join(ROOT, "segments", "shared"))
import records                                            # noqa: E402
spec = importlib.util.spec_from_file_location("train_data", os.path.join(ROOT, "segments", "train", "data", "make_data.py"))
TM = importlib.util.module_from_spec(spec)
spec.loader.exec_module(TM)                                # the train's: cm, hundredths, triangles, put
import train as T                                          # noqa: E402
import train_night as N                                    # noqa: E402
import callouts as CO                                      # noqa: E402

OUT = os.path.join(HERE, "scene.bin")
CLIPS = os.path.join(HERE, "clips.bin")
put, cm = TM.put, TM.cm

def kinds_renumbered(kind):
    """The night's kinds (40..56) from 60, but its platform, which is the train's (40) as it is."""
    k = kind.astype(int).copy()
    night = (k >= 40) & (k <= 56)
    k[night] = k[night] - 40 + 60
    k[kind == N.K_PLATFORM] = 40
    return k

def tri_keys(V):
    return [bytes(r) for r in np.round(V.reshape(len(V), 9) * 1000).astype(np.int64)]

def main():
    N.install()
    N.set_light(0.0)
    S = T.scene()                                          # the night's world (the drops made)
    V, NN, M = S["V"], S["N"], S["M"]
    kind, alb, par = S["mats"]
    small_new, lod, iz = S["small"].copy(), S["lod"].copy(), S["iz"].copy()
    nocull = np.zeros(len(kind), int)
    for m in N.EXTRA["nocull"]:
        nocull[int(m)] = 1
    poles = list(T.POLES)                                  # (the night's: past the town's edge dropped)
    extra = dict(N.EXTRA)
    screens = list(N.SCREENS)
    # the ride's own world, as it was (render_old's)
    saved = (T.STATIC, dict(T.HOOKS), T.scene, T.visible, T.TOWN0[0])
    T.STATIC = {}
    T.HOOKS.clear()
    T.scene, T.visible, T.TOWN0[0] = N.ORIG_SCENE, N.ORIG_VISIBLE, 1e9
    O = T.scene()
    T.STATIC, T.scene, T.visible, T.TOWN0[0] = saved[0], saved[2], saved[3], saved[4]
    T.HOOKS.update(saved[1])
    okind, oalb = O["mats"][0], O["mats"][1]
    used = int(O["M"].max()) + 1                           # (the other train's come after: not the world's)
    assert (okind[:used] == kind[:used]).all() and (oalb[:used] == alb[:used]).all(), "the old world's materials are the new's first"
    # the union: the new world's triangles (old too where the old has them), then the old's the new dropped
    newk = tri_keys(V)
    oldk = tri_keys(O["V"])
    inold = set(oldk)
    innew = set(newk)
    old_flag = np.array([k in inold for k in newk], int)
    only_old = np.array([k not in innew for k in oldk])
    small_old_of = dict(zip(oldk, O["small"]))
    Vw = np.concatenate([V, O["V"][only_old]])
    Nw = np.concatenate([NN, O["N"][only_old]])
    Mw = np.concatenate([M, O["M"][only_old]])
    flags_old = np.concatenate([old_flag, np.ones(only_old.sum(), int)])
    flags_new = np.concatenate([np.ones(len(V), int), np.zeros(only_old.sum(), int)])
    so = np.array([small_old_of.get(k, False) for k in newk] + list(O["small"][only_old]), int)
    sn = np.concatenate([small_new, np.zeros(only_old.sum(), bool)]).astype(int)
    lodw = np.concatenate([lod, O["lod"][only_old]])
    izw = np.concatenate([iz, O["iz"][only_old]])
    print("world: %d new (%d of them old), %d old only" % (len(V), old_flag.sum(), only_old.sum()))
    TM.triangles(Vw, Nw, Mw, "w_")
    put("w_zlo", np.floor(Vw[..., 2].min(1)).astype(int))
    put("w_zhi", np.ceil(Vw[..., 2].max(1)).astype(int))
    put("w_xmid", np.round(Vw[..., 0].mean(1)).astype(int))
    put("w_small_new", sn)
    put("w_small_old", so)
    put("w_lod", lodw.astype(int))
    put("w_iz", np.where(izw < -1e8, -999999, np.round(izw)).astype(int))
    put("w_old", flags_old)
    put("w_new", flags_new)
    # what moves: our own train (its nose at 0, toward +z), the drone (at 0, its nose toward +z), the airship
    # (its model at its scale, about its centre)
    for name, pre, scale in (("own", "o_", 1.0), ("drone", "d_", 1.0), ("blimp", "b_", N.BSC)):
        a = extra[name]
        TM.triangles(a[0] * scale, a[1], a[2], pre)
    put("m_kind", kinds_renumbered(kind))
    put("m_alb", np.round(alb * 1000).astype(int))
    put("m_par", np.round(par * 100).astype(int))
    put("m_nocull", nocull)
    put("poles", np.round(np.array(poles) * 100).astype(int))
    put("wires", night_wires(poles))
    # the street lights (bent, cm)
    put("lamps", cm(extra["lamps"] + np.array([N.BX, 0.0, 0.0])))
    # the screens: centre (unbent, cm), right and up (thousandths), width, height (cm), demo, kind (0 clip, 1 banner)
    put("screens", np.array([list(cm(s["c"])) + list(np.round(s["r"] * 1000).astype(int)) + list(np.round(s["u"] * 1000).astype(int)) +
                             [round(s["w"] * 100), round(s["h"] * 100), s["demo"], 0 if s["kind"] == "clip" else 1] for s in screens]))
    stadium_lights()
    airship()
    balloons()
    fireworks()
    flashes()
    stars()
    keyframes()
    pictures(screens)
    with open(OUT, "wb") as f:
        for name, a in TM.ARRAYS:
            f.write(records.record(name, a))
    for name, a in TM.ARRAYS:
        print(name, a.shape, a.min() if a.size else "", a.max() if a.size else "")
    clips(screens)

def night_wires(poles):
    """The overhead line over both tracks from the ride's end to well past the station, as the prototype's wires()
    draws it (fixed in the world: nodes every 5 m), and the farm road's power lines short of the town: x0 y0 z0 x1 y1
    z1 (cm), radius (mm), shine (thousandths), kind (0 the wires, 1 a dropper, 2 a power line)."""
    segs = []
    zs = np.arange(N.Z0 - 100.0, N.ZS + 1100.0, 5.0)
    for xw in (0.0, T.X2):
        span = np.mod(zs, 50.0) / 50.0
        stagger = 0.22 * (1 - 2 * np.abs(np.mod(zs / 50.0, 2) - 1))
        P1 = np.stack([xw + stagger - 0.11 + T.bend(zs), np.full_like(zs, 5.1), zs], 1)
        P2 = np.stack([xw + T.bend(zs), 6.6 - 0.9 * 4 * span * (1 - span), zs], 1)
        for P, r, sh in ((P1, 0.012, 0.02), (P2, 0.010, 0.015)):
            for k in range(len(zs) - 1):
                segs.append(list(cm(P[k])) + list(cm(P[k + 1])) + [round(r * 1000), round(sh * 1000), 0])
        for k in range(len(zs)):
            segs.append(list(cm(P1[k])) + list(cm(P2[k])) + [6, 20, 1])
    for zp0, zp1 in zip(poles[:-1], poles[1:]):
        zz = np.linspace(zp0, zp1, 9)
        u = (zz - zp0) / (zp1 - zp0)
        for dx in (-1.1, 0.0, 1.1):
            yy = T.LAND + 9.85 - 0.9 * 4 * u * (1 - u)
            P = np.stack([np.full_like(zz, T.MID - 19 + dx) + T.bend(zz), yy, zz], 1)
            for k in range(8):
                segs.append(list(cm(P[k])) + list(cm(P[k + 1])) + [12, 20, 2])
    return np.array(segs)

def stadium_lights():
    """The stadium's lights (stadium.json, their frame, cm; colours thousandths; phases thousandths of a radian)."""
    js = N._AU["json"]
    L = js["lights"]
    put("search", np.array([cm(l["pos"]).tolist() + [round(c * 1000) for c in l["rgb"]] + [round(l.get("power", 1.0) * 1000), round(l["phase"] * 1000)]
                            for l in L if l["type"] == "search"]))
    put("spots", np.array([cm(l["pos"]).tolist() + [round(c * 1000) for c in l["rgb"]] + [round(l["phase"] * 1000)]
                           for l in L if l["type"] == "spot"]))
    put("lasers", np.array([cm(l["pos"]).tolist() + [round(c * 1000) for c in l["rgb"]] + [l["n"], l["spread"]]
                            for l in L if l["type"] == "lasers"]))
    put("aud_c", cm(N.AUD_C))

def airship():
    """Its running lights about its centre (cm, at its scale) and kinds (0 warm, 1 the screen's frame, 2 a beacon);
    the up-lights (their source in the stadium's frame, cm; their phase, thousandths)."""
    pts, kinds = N.BLIMP_LIGHTS
    put("blimp_lights", np.concatenate([cm(pts * N.BSC), kinds[:, None]], 1))
    put("uplights", np.array([list(cm(src)) + [round(ph * 1000)] for src, ph in N.UPLIGHTS]))

def balloons():
    """900 balloons: where each starts (bent, cm), its rise (mm a frame), colour (thousandths), sway's phase
    (thousandths of a radian)."""
    N.balloons(np.zeros((2, 2, 3)), N.NightCam(0.0, 2, 2), np.zeros((2, 2)), 0.0, 0.0)   # (makes them)
    B = N._BAL
    put("balloons", np.concatenate([cm(B["p"]), np.round(B["v"] * 1000)[:, None], np.round(B["c"] * 1000),
                                    np.round(B["ph"] * 1000)[:, None]], 1).astype(int))

def fireworks():
    """The shells, for every burst the film can show (each starts on the first beat of a second bar, 24 frames
    apart, back from the hand-off): per burst, two shells: centre (bent, cm), colour (thousandths), 150 ways out
    (unit, thousandths)."""
    starts = [N.F_HAND - 24 * k for k in range(0, 40) if N.F_HAND - 24 * k >= 0]
    rows, ways = [], []
    for s in starts:
        rng = np.random.default_rng(int(s) * 7 + 3)
        for _ in range(2):
            c0 = N.AUD_C + np.array([N.BX + rng.uniform(-120, 120), rng.uniform(140, 230), rng.uniform(-120, 120)])
            col = np.array([[1, 0.4, 0.2], [0.3, 0.6, 1], [1, 0.9, 0.3], [0.9, 0.3, 1], [0.3, 1, 0.5]][rng.integers(5)])
            d = rng.normal(size=(150, 3))
            d /= np.linalg.norm(d, axis=1, keepdims=True)
            rows.append([s] + list(cm(c0)) + list(np.round(col * 1000).astype(int)))
            ways.append(np.round(d * 1000).astype(int))
    put("fw_shells", np.array(rows))
    put("fw_ways", np.concatenate(ways))

def flashes():
    """The photo flashes, every frame's 36 (as the prototype draws them, each one frame): the place across the bowl
    (r thousandths, the angle in thousandths of a radian), its height (cm), its colour's draw (thousandths)."""
    rows = []
    for f in range(0, N.F_HAND + 1):
        rng = np.random.default_rng(int(round(f)) * 131 + 7)
        n = 36
        r = rng.uniform(0.12, 0.93, n)
        a = rng.uniform(0, 2 * np.pi, n)
        z = np.where(r < 0.62, 1.7, 1.7 + (r - 0.62) / 0.33 * 28.0)
        hue = rng.uniform(0, 1, (n, 3)) ** 2
        for i in range(n):
            rows.append([f, round(r[i] * 1000), round(a[i] * 1000), round(z[i] * 100)] + list(np.round(hue[i] * 1000).astype(int)))
    put("flashes", np.array(rows))

def stars():
    """The sky's stars as the prototype finds them (a cell of 1/900 of a radian of azimuth and of the way's height
    part, where its hash passes 0.9975, above 0.06): each star's cell's middle as a way (ten-thousandths) and its
    brightness (thousandths)."""
    out = []
    cy = np.arange(54, 900)
    for cx0 in range(-2830, 2830, 400):
        cx = np.arange(cx0, min(cx0 + 400, 2830))
        X, Y = np.meshgrid(cx.astype(float), cy.astype(float))
        h = T.hash2(X, Y)
        el_ok = (Y + 1) / 900 > 0.06
        sel = (h > 0.9975) & el_ok & (Y / 900 > 0.06 - 1 / 900)
        for x, y, hh in zip(X[sel], Y[sel], h[sel]):
            az, el = (x + 0.5) / 900, (y + 0.5) / 900
            if not (-math.pi <= az < math.pi) or el >= 1:
                continue
            r = math.sqrt(1 - el * el)
            out.append([round(r * math.sin(az) * 10000), round(el * 10000), round(r * math.cos(az) * 10000), round((hh - 0.9975) * 400 * 1000)])
    put("stars", np.array(out))

def keyframes():
    """The flight's design: the path's keyframes (film, unbent, cm), and the places it is set by (cm)."""
    put("path", cm(np.array(N.PATH)))
    named = dict(view=N.VIEW, drone_hover=N.DRONE_HOVER, drone_from=N.DRONE_FROM, handoff=N.HANDOFF,
                 hand_look=N.HAND_LOOK, read=N.to_film(N.READ_AU), orbit0=N.to_film(N.ORBIT[0]),
                 orbit1=N.to_film(N.ORBIT[-1]), turn=N.to_film(N.APPROACH[2]), drone_away=N.to_film(np.array([-150.0, -150.0, 190.0])),
                 blimp_au=N.BLIMP_AU, blimp_screen_au=N.BLIMP_SCREEN_AU)
    put("places", np.array([cm(v) for v in named.values()]))
    print("places:", list(named.keys()))
    print("prototype: F_HAND %d, NCH %d, F_READ %d, ZS %.2f, D_HAND %.2f, D_READ %.2f, D_ORB0 %.2f, D_TURN %.2f" %
          (N.F_HAND, N.NCH, N.F_READ, N.ZS, N.D_HAND, N.D_READ, N.D_ORB0, N.D_TURN))

def pictures(screens):
    """The printed pictures: the board's dots (0 dark, 1 orange, 2 amber), each screen's banner or caption, the
    stadium's marquee, the airship's words and the volume knob's printed parts."""
    put("led", N.LED.astype(int))
    for s in screens:
        i = s["demo"]
        if s["kind"] == "banner":
            ban, _ = CO.banner(i, 256, 96)
            put("banner_%d" % i, np.round(np.clip(ban, 0, 1) * 1000).astype(int).reshape(96, 256 * 3))
    for i in clip_demos(screens):
        cap, m = CO.caption(i, 160, 30)
        put("cap_%d" % i, np.round(np.clip(cap, 0, 1) * 1000).astype(int).reshape(30, 160 * 3))
        put("capm_%d" % i, np.round(m * 100).astype(int))
    put("marquee", CO.marquee().astype(int))
    # the parodies' stand-ins: each entry's (-1 a clip, else plasma, tunnel, rotozoom, stars, rings) and their tables
    put("si_kind", np.array([CO.standin_kind(i) for i in range(len(CO.ENTRIES))]))
    for name, a in CO.standin_tables().items():
        put(name, np.asarray(a).astype(int))
    put("blimp_text", np.round(N.BLIMP_TEXT * 100).astype(int))
    vol_parts()

def vol_parts():
    """The volume knob's printed parts (train_night.py's vol_tex, taken apart): the ground with the scale 0-10 and
    the knob's face (sRGB 0..255, as drawn), and masks (coverage 0..255) of what CARDIAC colours: the 11's tick and
    figure, VOL, the 10 and the 11 beside it."""
    from PIL import Image, ImageDraw, ImageFont
    W_, H_ = 1000, 560
    F = lambda n: ImageFont.truetype(CO.HEL, n, index=1)
    cx, cy, R = 290, 215, 130
    im = Image.new("RGB", (W_, H_), (0, 0, 0))
    d = ImageDraw.Draw(im)
    for y in range(H_):
        d.line([(0, y), (W_, y)], fill=(int(30 + 30 * y / H_), 6, int(48 + 40 * y / H_)))
    m11 = Image.new("L", (W_, H_), 0)
    d11 = ImageDraw.Draw(m11)
    for v_ in range(12):
        a = math.radians(-135 + 27 * v_)
        x0, y0 = cx + math.sin(a) * R * 1.08, cy - math.cos(a) * R * 1.08
        x1, y1 = cx + math.sin(a) * R * (1.32 if v_ >= 10 else 1.22), cy - math.cos(a) * R * (1.32 if v_ >= 10 else 1.22)
        fs = F(120 if v_ == 11 else (76 if v_ == 10 else 36))
        rr = 1.8 if v_ == 11 else (1.7 if v_ == 10 else 1.5)
        tx, ty = cx + math.sin(a) * R * rr, cy - math.cos(a) * R * rr
        t = str(v_)
        if v_ == 11:
            d11.line([(x0, y0), (x1, y1)], fill=255, width=9)
            d11.text((tx - d11.textlength(t, font=fs) / 2, ty - fs.size * 0.6), t, font=fs, fill=255)
        else:
            col = (235, 232, 225)
            d.line([(x0, y0), (x1, y1)], fill=col, width=9 if v_ >= 10 else 5)
            d.text((tx - d.textlength(t, font=fs) / 2, ty - fs.size * 0.6), t, font=fs, fill=col)
    d.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(20, 20, 22), outline=(120, 120, 126), width=6)
    put("vol_base", np.array(im).astype(int).reshape(H_, W_ * 3))
    put("vol_m11", np.array(m11).astype(int))
    for name, s, xy, size in (("vol_mvol", "VOL", (610, 120), 150), ("vol_m10", "10", (600, 270), 200), ("vol_m11b", "11", (556, 175), 390)):
        mk = Image.new("L", (W_, H_), 0)
        ImageDraw.Draw(mk).text(xy, s, font=F(size), fill=255)
        put(name, np.array(mk).astype(int))

def clip_demos(screens, footage=False):
    """The demos the screens show: the screens' and those the airship's screen cycles through (with footage: only the
    clips, not the parodies' stand-ins)."""
    ds = {s["demo"] for s in screens if s["kind"] == "clip"}
    for f in range(0, N.F_HAND + 1):
        seg = int((f + 6) // 30) % 3
        if seg:
            ds.add((int((f + 6) // 90) * 2 + seg) % len(CO.ENTRIES))
    return sorted(d for d in ds if not footage or CO.standin_kind(d) < 0)

def clips(screens):
    """The footage, as bytes: the count of demos, each demo's number, then each one's 60 frames of 120 rows of 160
    pixels' RGB."""
    ds = clip_demos(screens, footage=True)
    with open(CLIPS, "wb") as f:
        f.write(struct.pack("<I", len(ds)))
        for i in ds:
            f.write(struct.pack("<I", i))
        for i in ds:
            fr = CO._frames(i)
            assert fr is not None and len(fr) == 60, "demo %d has no footage" % i
            f.write(np.round(fr * 255).astype(np.uint8).tobytes())
    print("clips:", ds)

if __name__ == "__main__":
    main()
