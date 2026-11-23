"""Design prototype (the departure): the ride now starts at a standstill in a station, by day. The cab
waits at the end of the platform under the canopy, the departure board beside it; it pulls away, Shinkansen-quick,
out from under the canopy and along the viaduct into the city, and reaches the ride's speed, place and heading
exactly at train.py's frame 0, which carries on as before. It mirrors the ending (the night arrival).

    0-11     standing, the board in view
    12-83    pulling away: from rest to the ride's 7 m a frame at a steady push (252 m, 6 s)
    84-      train.py's ride, its frame k at our frame 84 + k

It uses train.py's renderer through its hooks (unset, train.py renders exactly as without them), the station and its
board from models/build_night_models.py, and a stretch of town of its own behind the ride's start (its
own random numbers, so the ride's world is the same).

    python train_depart.py OUT.mp4 [--size 1536x768] [--jobs 12] [--frames A:B]   # SS=2 for the 2x2 samples
    python train_depart.py --stills K,K,K OUTDIR [--size 768x384]
"""
import math
import os
import subprocess
import sys

import numpy as np

import train as T

HOLD, ACC = 12, 72                                        # standing, then pulling away (frames)
D = HOLD + ACC                                            # our frame of the ride's frame 0
Z_START = -T.SPEED * ACC / 2                              # where the cab stands (-252 m)
PL0, PL1 = Z_START - 170.0, Z_START + 30.0                # the platforms (200 m), their end 30 m ahead of the cab
BOARD_Z = Z_START + 16.0                                  # the departure board, hanging over the left platform
X2, MID = T.X2, T.MID
TOWN_END = -140.0                                         # the ride's city starts here (train.py's lots)
TOWN_FROM = PL0 - 220.0

K_PLATFORM, K_LED, K_TUBE, K_FENCE = range(40, 44)

def ride_t(f):
    """The ride's time (train.py's frames) at our frame f: before 0 while departing."""
    return f - D

def cab(t):
    """The cab's place along the line at ride time t and how much it sways: standing until -ACC, then a steady push
    to the ride's speed at 0 (z = 0 there, as train.py's frame 0)."""
    if t >= 0:
        return t * T.SPEED, 1.0
    u = min(max((t + ACC) / ACC, 0.0), 1.0)
    return Z_START + T.SPEED * ACC * u * u / 2, u

# ---- the board ----------------------------------------------------------------------------------------------------

def led_bitmap():
    """The departure board's dots: the IBM VGA 8x16 font, 128 x 40: the train, its time, where it's going."""
    from PIL import Image, ImageDraw, ImageFont
    f = ImageFont.truetype(os.path.join(os.path.dirname(os.path.abspath(__file__)), "pcfonts", "Px437_IBM_VGA_8x16.ttf"), 16)
    im = Image.new("L", (128, 40), 0)
    d = ImageDraw.Draw(im)
    for i, s in enumerate(["14:03 NOZOMI 37", "FOR SHIN-OSAKA"]):
        d.text((0, 3 + i * 18), s, font=f, fill=255)
    a = (np.array(im) > 127).astype(np.int8)
    a[20:] *= 2
    return a

LED = led_bitmap()
LED_COL = {1: np.array([1.0, 0.62, 0.10]), 2: np.array([0.30, 0.95, 0.35])}
BOARD = dict(cx=-4.6, y0=4.25, z=BOARD_Z, w=9.6, h=2.9)

# ---- the world: the station and the town behind the ride's start -------------------------------------------------

def build_extra(g0, mats):
    """Everything as instanced chunks after the ride's (the boxes too, in a chunk of their own), so the ride's
    triangles keep their places in the list."""
    g = g0
    gb = T.Geo()
    rng = np.random.default_rng(1872)                     # the year Japan's first railway opened
    M = T.model
    conc = mats.add(T.K_CONC, (0.55, 0.55, 0.53), (6.0,))
    steel = mats.add(T.K_METAL, (0.42, 0.43, 0.45))
    roles = {"platform": mats.add(K_PLATFORM, (0.52, 0.51, 0.49)), "fence": mats.add(K_FENCE, (0.62, 0.64, 0.66)),
             "steel": steel, "canopy": mats.add(T.K_CONC, (0.70, 0.70, 0.68), (0.0,)),
             "tube": mats.add(K_TUBE, (1.0, 0.97, 0.9)),
             "signsmall": mats.add(T.K_SIGN, (0.1, 0.3, 0.8), (17.0,)),
             "buildwall": mats.add(T.K_BUILD, (0.60, 0.61, 0.63), (2.0, 3.0, 0.0, 0.3, 16.0, T.STREET)),
             "trim": conc, "unit": mats.add(T.K_ROOFTOP, (0.6, 0.6, 0.6)),
             "namesign": mats.add(T.K_SIGN, (0.95, 0.95, 0.95), (3.0, 1.0))}
    g.place(M("station"), (0.0, 0.0, PL0), 0.0, roles=roles)
    led = mats.add(K_LED, (0.02, 0.02, 0.02))
    g.place(M("led_board"), (BOARD["cx"], BOARD["y0"], BOARD_Z), 0.0,
            roles={"frame": mats.add(T.K_DARK, (0.03, 0.03, 0.035)), "led": led, "steel": steel})
    T.NOCULL[led] = True
    # the viaduct's parapets on from the platforms' end and behind them; the masts
    pm = mats.add(T.K_CONC, (0.58, 0.57, 0.54), (10.0,))
    for z0 in np.arange(TOWN_FROM, -120.0, 20.0):
        if PL0 - 2 < z0 + 20 and z0 < PL1 + 2:
            continue
        for xa, xb in ((MID - 5.95, MID - 5.65), (MID + 5.65, MID + 5.95)):
            gb.box((xa, -0.6, z0), (xb, 1.15, z0 + 20), pm)
    if gb.V:
        Vb, Nb, Mb = gb.arrays()
        g.chunks.append((Vb, Nb, Mb, 0, -500.0))
    mm = mats.add(T.K_METAL, (0.40, 0.41, 0.43))
    mbase = mats.add(T.K_CONC, (0.50, 0.49, 0.47), (0.0,))
    for z in np.arange(-150.0, TOWN_FROM, -50.0):
        if PL0 - 5 < z < PL1 + 8:
            continue
        g.place(M("mast"), (0.0, 0.0, z), 0.0, roles={"steel": mm, "conc": mbase})
    # the town: houses and blocks on lots along streets beside the viaduct, as the ride's city, up to where it starts
    siding = [(0.74, 0.71, 0.64), (0.66, 0.62, 0.55), (0.78, 0.77, 0.74), (0.58, 0.55, 0.50), (0.70, 0.62, 0.50)]
    tiles = [(0.10, 0.11, 0.14), (0.16, 0.13, 0.12), (0.08, 0.10, 0.16), (0.20, 0.20, 0.21)]
    pal = [(0.55, 0.53, 0.50), (0.62, 0.56, 0.47), (0.44, 0.43, 0.42), (0.72, 0.71, 0.68), (0.60, 0.60, 0.55)]
    acu = mats.add(T.K_ROOFTOP, (0.70, 0.70, 0.68))
    def clear(x0, x1, z0, z1):
        # not on the station's building (beyond the right platform) nor its forecourt
        return not (x1 > X2 + 10 and x0 < X2 + 52 and z1 > PL0 + 30 and z0 < PL0 + 180)
    for side in (-1, 1):
        x = 13.0
        while x < 170:
            near = x < 40
            for k in range(2):
                dep = float(rng.uniform(10, 14))
                z = TOWN_FROM + float(rng.uniform(0, 8))
                while z < TOWN_END - 6:
                    fr = float(rng.uniform(8, 12))
                    xa, xb = (MID - x - dep, MID - x) if side < 0 else (MID + x, MID + x + dep)
                    if clear(xa, xb, z, z + fr) and rng.random() > 0.07:
                        r_ = rng.random()
                        if r_ < (0.6 if near else 0.7):
                            nm = "house_%d" % int(rng.integers(8))
                            wm = mats.add(T.K_CHOUSE, siding[int(rng.integers(len(siding)))],
                                          (float(rng.uniform(0, 100)), T.STREET + 6.0, T.STREET,
                                           float(rng.uniform(2.6, 3.8)), float(rng.integers(0, 3))))
                            r = {"wall": wm, "roof": mats.add(T.K_TILE, tiles[int(rng.integers(len(tiles)))]),
                                 "tin": mats.add(T.K_SHED, (0.15, 0.30, 0.55)), "trim": conc, "rail": conc,
                                 "unit": acu, "solar": mats.add(T.K_SOLAR, (0.03, 0.05, 0.12))}
                            g.place(M(nm), ((xa + xb) / 2, T.STREET, z + fr / 2), float(rng.integers(4)) * math.pi / 2,
                                    (0.95, 1.0, 0.95), r)
                        else:
                            fl = int(rng.integers(4, 11))
                            kb = min(3, max(0, (fl - 4) // 2))
                            ln = fr + float(rng.uniform(4, 12))
                            sy = fl / (5, 7, 9, 12)[kb]
                            h = T.STREET + 3.2 * fl
                            wm = mats.add(T.K_BUILD, pal[int(rng.integers(len(pal)))],
                                          (float(rng.choice([0, 1, 3])), float(rng.uniform(2.4, 3.4)),
                                           float(rng.integers(4)), float(rng.uniform(0, 1)), h, T.STREET))
                            g.place(M("block_%d" % kb), ((xa + xb) / 2, T.STREET, z + ln / 2), float(rng.integers(4)) * math.pi / 2,
                                    (1.0, sy, 1.0), {"wall": wm, "trim": conc, "rail": conc, "unit": acu, "tank": steel})
                            fr = ln
                    z += fr + 1.0
                x += dep + 1.0
            x += 6.5

# the day's shadows behind the ride's start: a second map, used only there (the ride never sees it)
class TwoMaps:
    def __init__(self, a, b, split):
        self.a, self.b, self.split = a, b, split
    def lookup(self, P, mask, D=None, bias=0.35):
        behind = P[..., 2] < self.split
        la = self.a.lookup(P, mask & ~behind)
        lb = self.b.lookup(P, mask & behind)
        return np.where(behind, lb, la)

def install():
    if T.HOOKS.get("cab"):
        return
    T.HOOKS["cab"] = cab
    T.HOOKS["build"] = build_extra
    T.HOOKS["materials"] = materials
    S = T.scene()
    if not isinstance(S["smap_day"], TwoMaps):
        far = S["V"][..., 2].max(1) < -60
        m2 = T.SMap(T.DAY.sun, (MID, 0.0, (TOWN_FROM - 60) / 2), -150, 150, -260, 260, 0.1, 0.1)
        m2.raster(S["V"][far])
        S["smap_day"] = TwoMaps(S["smap_day"], m2, -100.0)

# ---- the station's materials ---------------------------------------------------------------------------------------

def materials(kind, P, Nn, Dn, pr, alb, emit, spec, foot, t):
    m = kind == K_LED
    if m.any():
        p = P[m]
        b = BOARD
        x = p[:, 0]
        u = np.where(Dn[m][:, 2] < 0, (b["cx"] + b["w"] / 2) - x, x - (b["cx"] - b["w"] / 2))
        u = u - (b["w"] - 128 * 0.0725) / 2
        v = (b["y0"] + b["h"]) - p[:, 1]
        col, row = np.floor(u / 0.0725), np.floor(v / 0.0725)
        fu, fv = u / 0.0725 - col, v / 0.0725 - row
        ok = (col >= 0) & (col < 128) & (row >= 0) & (row < 40)
        code = np.where(ok, LED[np.clip(row, 0, 39).astype(int), np.clip(col, 0, 127).astype(int)], 0)
        fw = foot[m] / 0.0725
        dot = T.fband(fu, 0.5, 0.36, fw) * T.fband(fv, 0.5, 0.36, fw)
        c = np.where((code == 1)[:, None], LED_COL[1], LED_COL[2])
        lit = code > 0
        emit[m] = c * (dot * np.where(lit, 4.0, 0.03))[:, None] + c * (lit * 0.25 * np.clip(fw - 0.5, 0, 1))[:, None]
        alb[m] = 0.02
    m = kind == K_PLATFORM
    if m.any():                                           # the tactile strip (yellow) and the white edge line
        p = P[m]; top = Nn[m][:, 1] > 0.5
        x = p[:, 0]
        e = np.where(x < MID, np.abs(x - (-2.6)), np.abs(x - (X2 + 2.6)))
        edge = np.where(x < MID, np.abs(x - (-1.85)), np.abs(x - (X2 + 1.85)))
        strip = T.fband(e, 0.0, 0.16, foot[m]) * top
        white = T.fband(edge, 0.0, 0.08, foot[m]) * top
        a = alb[m] * (0.85 + 0.3 * T.fbm(p[:, 0] * 0.8, p[:, 2] * 0.8, 2))[:, None]
        a = a * (1 - strip[:, None]) + np.array([0.85, 0.65, 0.05]) * strip[:, None]
        alb[m] = a * (1 - white[:, None]) + np.array([0.8, 0.8, 0.78]) * white[:, None]
    m = kind == K_TUBE
    if m.any():                                           # the canopy's light tubes, on by day too
        emit[m] = alb[m] * 2.5

# ---- running it ---------------------------------------------------------------------------------------------------

NFR = D + 36                                              # the departure and the ride's first 3 s

def frame_bytes(args):
    k, W, H = args
    install()
    t = ride_t(k)
    SS = T.SS
    img = T.render(float(t), W * SS, H * SS, overlay=False)
    if SS > 1:
        img = img.reshape(H, SS, W, SS, 3).mean((1, 3))
    if not os.environ.get("NOTEXT"):
        img = T.overlays(img, float(t), W, H)
    return np.clip(img, 0, 255).astype(np.uint8).tobytes()

def main():
    args = sys.argv[1:]
    def opt(name, default):
        if name in args:
            i = args.index(name); v = args[i + 1]; del args[i:i + 2]; return v
        return default
    W, H = map(int, opt("--size", "1536x768").split("x"))
    jobs = int(opt("--jobs", "10"))
    stills = opt("--stills", None)
    rng_ = opt("--frames", None)
    print("cab at %.0f m standing, platforms %.0f..%.0f, board at %.0f; the ride's frame 0 at our %d; %d frames"
          % (Z_START, PL0, PL1, BOARD_Z, D, NFR), flush=True)
    from multiprocessing import Pool
    if stills:
        from PIL import Image
        outdir = args[0]
        os.makedirs(outdir, exist_ok=True)
        ks = [int(k) for k in stills.split(",")]
        with Pool(min(jobs, len(ks))) as pool:
            for k, b in zip(ks, pool.imap(frame_bytes, [(k, W, H) for k in ks])):
                Image.fromarray(np.frombuffer(b, np.uint8).reshape(H, W, 3)).save(f"{outdir}/d{k:04d}.png")
        return
    out = args[0]
    a, b = (0, NFR) if not rng_ else map(int, rng_.split(":"))
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}",
                           "-r", str(T.FPS), "-i", "-", "-c:v", "libx264", "-profile:v", "main", "-level", "4.0",
                           "-bf", "0", "-g", "12", "-crf", "17", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out],
                          stdin=subprocess.PIPE)
    with Pool(jobs) as pool:
        for i, bts in enumerate(pool.imap(frame_bytes, [(k, W, H) for k in range(a, b)])):
            ff.stdin.write(bts)
            if i % 24 == 0:
                print("frame", a + i, flush=True)
    ff.stdin.close(); ff.wait()
    print(b - a, "frames,", round((b - a) / T.FPS, 1), "s")

if __name__ == "__main__":
    main()
