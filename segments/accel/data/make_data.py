"""The accelerated drive's data, printed once from the design (prototypes/accel.py, look A): the street's grade, the
script run on to frame 183, the window kinds, the palette (the base colours the machines fog and mix, in tenths of a
16-bit step), the wall texture's bays (texel ids of the palette), the street's palette (asphalt and walk under each
level of lamplight, the stripe at each step of its dashes), the pools' falloffs, the dither's thresholds, the glow's
profile, and the traffic's pictures at their sizes.

    python make_data.py        (with numpy and PIL)

Colours: a channel is in tenths of a 16-bit step: red and blue 0-310 (31 steps of five bits), green 0-630 (63 of
six); 8-bit v is v 310 / 255 or v 630 / 255, rounded. The wall texture is 16-bit (RGB565) as a card's textures were:
each texel's channels whole steps (tens).
"""
import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "../../../prototypes"))
sys.argv = sys.argv[:1]
import accel  # noqa: E402  (the design: its hill, steering, colours and car pictures)

DRIVE = os.path.join(HERE, "../../drive/data")
FRAMES = accel.FRAMES


def put(name, lines):
    open(os.path.join(HERE, name), "w").write("\n".join(lines) + "\n")


def tenths(rgb):
    return [round(rgb[0] * 310 / 255), round(rgb[1] * 630 / 255), round(rgb[2] * 310 / 255)]


def texel(rgb):
    """16-bit: whole steps, in tenths."""
    return [10 * round(min(255, max(0, rgb[0])) * 31 / 255), 10 * round(min(255, max(0, rgb[1])) * 63 / 255),
            10 * round(min(255, max(0, rgb[2])) * 31 / 255)]


def hash2(a, b):
    return ((((a * 73856093) ^ (b * 19349663)) * 2654435761) & 0xffff) / 65536.0


# ---- the grade: the street's height every 2 units along (eighths), as the road's plan --------------------------
put("grade.txt", [str(round(8 * float(accel.hill(2 * i)))) for i in range(400)])
# the street's plan, its middle across every 2 units (eighths), as the drive's road.txt but twisting all the way
put("road.txt", [str(round(8 * float(accel.road(2 * i)))) for i in range(400)])
# the blocks (the drive's hash, run on to 30) and the traffic (two more slow cars and more oncoming), as the drive's
put("blocks.txt", [" ".join(map(str, b)) for b in accel.BLOCKS])
# (a card holds at most 999: the furthest slow car starts a quarter nearer)
put("cars.txt", [" ".join(map(str, [c[0], min(c[1], 999)] + list(c[2:]))) for c in accel.CARS])

# ---- the tables the camera's path and our car's lane are worked out from on CARDIAC (aphase, aswerve, alane) ------
# the wobble, 0.4 sin(1.7 t), as 3.2 sin in 216ths of an eighth: a quarter turn of a 1000-step turn (251 entries)
put("lanesine.txt", [str(round(3.2 * 216 * math.sin(2 * math.pi * i / 1000))) for i in range(251)])
# the swerve past a slow car, 32 times the design's smooth(26, 14, u) smooth(-10, -2, u), in eighths: whole eighths
# and 216ths over. Entries 0-12: one smooth rising, 8 a a (18 - a) / 216 (a = 26 - u); 13-21: the other falling,
# 27 b b (12 - b) / 216 (b = u + 10)
put("laneswerve.txt", ["%d %d" % divmod(8 * a * a * (18 - a), 216) for a in range(13)] +
    ["%d %d" % divmod(27 * b * b * (12 - b), 216) for b in range(9)])
# the jet's pictures (the design's jet_picture), one for each frame it is in the strip of sky: its size from k = f -
# 103 as the machines work it out (half its span 64 + 4 k k / 9 16ths; width that / 8, height 7 that / 128 pixels);
# each pixel's colour in tenths of a step (8-bit v SCALE / 255, rounded down) and its cover in 32nds (rounded)
jet_lines = []
for k in range(1, 37):
    size = 64 + 4 * k * k // 9
    jw, jh = max(2, size // 8), max(1, 7 * size // 128)
    pic = accel.jet_picture(jw, jh, 103 + k)
    jet_lines.append(f"{k} {jw} {jh}")
    for y in range(jh):
        jet_lines.append(" ".join(f"{int(pic[y, x, 0]) * 310 // 255} {int(pic[y, x, 1]) * 630 // 255} {int(pic[y, x, 2]) * 310 // 255} {round(pic[y, x, 3] * 32 / 255)}" for x in range(jw)))
put("jet.txt", jet_lines)
# the sparks where our car comes down (the design's spark_set, its random numbers as it draws them): at the landing
# (frame 174) 60 from each end, at the bounce (180) 40 from the back; each: its frame, its end (64ths along from
# the car's middle), its speed across, along and up (64ths a second) and its heat (thousandths); then how bright a
# spark is t frames old, 1000 (1 - t / 7.2)^1.5 (t 0-7)
spark_lines = []
count = 0
for f0, n, seed, ends in ((174, 120, 1, (-1.7, 1.7)), (180, 40, 2, (-1.7,))):
    for e in ends:
        for (_, _, v, heat) in accel.spark_set(f0, n // len(ends), seed + 10 * count, None):
            spark_lines.append(f"{f0} {round(64 * e)} {round(64 * v[0])} {round(64 * v[1])} {round(64 * v[2])} {round(1000 * heat)}")
        count += n // len(ends)
spark_lines.append(" ".join(str(round(1000 * (1 - t / 7.2) ** 1.5)) for t in range(8)))
put("sparks.txt", spark_lines)

# ---- the window kinds: 0 unlit, 1-4 lit (warm to cool), 5-8 lit with blinds; the drive's window texture, a quarter
# of the lit windows with blinds drawn ----
windows = [int(l) for l in open(os.path.join(DRIVE, "windows.txt")) if l.strip()]
kinds = [0 if m < 5 else (m - 4) + (4 if hash2(i + 3, 7) < 0.25 else 0) for i, m in enumerate(windows)]
put("kinds.txt", [str(k) for k in kinds])

# ---- the palette ------------------------------------------------------------------------------------------------
palette, names = [], {}


def colour(c, name=None):
    c = list(c)
    if c in palette:
        k = palette.index(c)
    else:
        palette.append(c)
        k = len(palette) - 1
    if name:
        names[name] = k
    return k


WALL, LEDGE, UNLIT, LIT = accel.WALL, accel.LEDGE, accel.UNLIT, accel.LIT
# the wall texture: a bay (two units across, a floor three up) is 8 x 12 texels, a quarter unit each; its texels
# for each tone (0, 1), lit face or end wall in shade (0, 1) and window kind (0-8), and the top four rows of a
# block (a wall, then its ledge)
bays, roofs = [], []
for tone in (0, 1):
    for end in (0, 1):
        k_end = 0.62 if end else 1.0
        for kind in range(9):
            ids = []
            for ii in range(8):
                for jj in range(12):
                    s, y = (ii + 0.5) * 0.25, (jj + 0.5) * 0.25
                    grain = 1 + 0.10 * (hash2(ii * 3 + 1, jj * 7 + 5 * tone) - 0.5)
                    if ii < 2 or jj < 4:
                        c = WALL[tone] * grain
                    else:
                        if kind == 0:
                            w = UNLIT * (1.6 if y > 2.25 else 1.0)
                        else:
                            shade = 0.82 + 0.3 * (y - 1) / 2 + 0.12 * (hash2(ii * 7 + 3, 11) - 0.5)
                            blinds = kind >= 5 and jj % 2 == 0
                            w = LIT[(kind - 1) % 4] * shade * (0.7 if blinds else 1.0)
                        if ii == 2 or ii == 7 or jj == 4 or jj == 11:
                            w = w * 0.55 + UNLIT * 0.45
                        c = w
                    ids.append(colour(texel(c * k_end)))
            bays.append(" ".join(map(str, ids)))
        for part in (WALL, LEDGE):
            roofs.append(" ".join(str(colour(texel(part[tone] * (1 + 0.10 * (hash2(ii * 3 + 1, jj * 7 + 5 * tone) - 0.5)) * k_end)))
                                  for ii in range(8) for jj in range(12)))
put("bays.txt", bays)      # 36 lines (tone, end, kind), 96 ids each (ii 0-7, then jj 0-11)
put("roofs.txt", roofs)    # 8 lines (tone, end, wall then ledge), 96 ids each

# the street: asphalt and walk under each level of the lamps' light (pool 0-32, 32nds of A B), the stripe at each
# step of its dash (0-32), the edge line, the kerb, beyond, the sky
ASPHALT, POOL, STRIPE, EDGE, WALK, BEYOND = accel.ASPHALT, accel.POOL, accel.STRIPE, accel.EDGE, accel.WALK, accel.BEYOND
KERB = np.array([90, 88, 92], np.float32)
street = {}
street["asphalt"] = [colour(tenths(ASPHALT + (POOL - ASPHALT) * 0.7 * p / 32)) for p in range(33)]
street["walk"] = [colour(tenths(WALK * (1 + 0.5 * p / 32))) for p in range(33)]
street["stripe"] = [colour(tenths(ASPHALT + (STRIPE - ASPHALT) * d / 32)) for d in range(33)]
for n, c in (("edge", EDGE), ("kerb", KERB), ("beyond", BEYOND), ("sky", accel.SKY_LOW), ("post", accel.POST)):
    street[n] = [colour(tenths(c), n)]
put("street.txt", ["%s %s" % (n, " ".join(map(str, v))) for n, v in street.items()])
put("palette.txt", ["%d %d %d" % tuple(c) for c in palette])

# the pools' falloffs, 32nds: along (dz quarters from the lamp's place, 0-36), across (|off| sixteenths, 0-160)
put("pools.txt", [" ".join(str(round(32 * max(0.0, 1 - (dz / 36) ** 2))) for dz in range(37)),
                  " ".join(str(round(32 * max(0.0, 1 - ((o / 16 - 5.2) / 4) ** 2))) for o in range(161))])

# the march's steps, quarters ahead: from 3 units, each a 25th further (a quarter at least), to 160; 0 at the end
st, a = [], 12
while a <= 640:
    st.append(a)
    a += max(1, a // 25)
put("steps.txt", [" ".join(map(str, st + [0]))])
# the software half's colours: for each channel the drive's ten levels (8-bit), where each of levels 1-8 starts in
# tenths of a step (SCALE v / 255 rounded up), and for each level 0-8 and Bayer value B 0-15 the tenths from which the
# next level up shows: ((2 B + 1) SCALE (hi - lo) + 32 SCALE lo) // 8160 + 1
LEVELS = [[0, 12, 26, 38, 52, 70, 100, 150, 205, 255], [0, 12, 26, 36, 48, 62, 90, 140, 205, 255], [0, 16, 28, 40, 54, 70, 100, 150, 205, 255]]
soft_lines = []
for c, sc in enumerate((310, 630, 310)):
    lv = LEVELS[c]
    soft_lines.append(" ".join(map(str, lv)))
    soft_lines.append(" ".join(str(-(-sc * lv[k] // 255)) for k in range(1, 9)))
    for i in range(9):
        soft_lines.append(" ".join(str(((2 * b + 1) * sc * (lv[i + 1] - lv[i]) + 32 * sc * lv[i]) // 8160 + 1) for b in range(16)))
put("softq.txt", soft_lines)
# the software half's march: coarser, a tenth further each step (at least 2 quarters)
st, a = [], 12
while a <= 640:
    st.append(a)
    a += max(2, a // 10)
put("steps_soft.txt", [" ".join(map(str, st + [0]))])

# the street's cross-section: for each pixel's reach m (3-22: a / 32 + 3, 64ths) and |across| (64ths, 0-999), what
# lies there: -2 within m of the stripe's edges (8 either side), -1 within m of another edge (the edge line 364-380,
# the kerb 384-393.6, the walk to 576), else 0 asphalt, 1 the stripe, 2 the edge line, 3 the kerb, 4 the walk, 5 beyond
def cross(m, off):
    if abs(off - 8) <= m or abs(off + 8) <= m:
        return -2
    if any(abs(off - b) <= m for b in (364, 380, 384, 576)) or abs(10 * off - 3936) <= 10 * m:
        return -1
    if off < 8: return 1
    if off < 364: return 0
    if off < 380: return 2
    if off < 384: return 0
    if 10 * off < 3936: return 3
    if off < 576: return 4
    return 5
put("cross.txt", [" ".join(str(cross(m, off)) for off in range(1000)) for m in range(23)])

# the card's DAC: each level of red, green and blue as 8-bit (the step times 255 / 31 or / 63, rounded down)
put("dac.txt", [" ".join(str(v * 255 // 31) for v in range(32)), " ".join(str(v * 255 // 63) for v in range(64)), " ".join(str(v * 255 // 31) for v in range(32))])

# the 4 x 4 ordered dither: each threshold as tenths of a step, floor((10 b + 5) / 16)
B4 = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5]
put("dither.txt", [" ".join(str((10 * b + 5) // 16) for b in B4)])

# the glow by octagon steps: the same at rho = t / 32, t 0-192
put("glowt.txt", [" ".join(str(round(500 * (math.exp(-(t / 32) ** 2 / (2 * 0.35 ** 2)) + 0.35 / (1 + (t / 32) ** 2)))) for t in range(193))])
# the glow: exp(-rho^2 / 0.245) + 0.35 / (1 + rho^2), thousandths, at rho^2 = i / 16, i 0-576
put("glow.txt", [" ".join(str(round(1000 * (math.exp(-i / 16 / (2 * 0.35 ** 2)) + 0.35 / (1 + i / 16)))) for i in range(577))])

# the flares' and ghosts' falloffs: E[t] = 1000 exp(-t / 16), t 0-191 (and 0 past it); LN[v] = 16 ln(1000 / v),
# v 1-1000 (the step at which E falls to v), so a light's strength can be taken into its falloff's index
put("exp.txt", [" ".join(str(round(1000 * math.exp(-t / 16))) for t in range(192)),
                " ".join(str(round(16 * math.log(1000 / v))) for v in range(1, 1001))])
# the ghosts' profiles, 500ths, at rho^2 = i / 16 (rho the distance over the ghost's radius), i 0-35: a disc
# (soft over its last 1.5 of 20 pixels, at 0.6) and a ring (at 0.85 of the radius, 0.22 of it each way)
put("ghosts.txt", [" ".join(str(round(500 * min(1.0, max(0.0, (1 - math.sqrt(i / 16)) * 20 / 1.5)) * 0.6)) for i in range(36)),
                   " ".join(str(round(500 * max(0.0, 1 - abs(math.sqrt(i / 16) - 0.85) / 0.22))) for i in range(36))])

# ---- the traffic's pictures: each body (from behind, from the front) drawn as the design draws it (144 x 56) and
# made smaller (PIL's bilinear) to every width 2-64, every 4 to 128, every 8 to 256, every 16 to 352 ----------------
cars = accel.CARS
CODES = [368, 884, 888, 476, 757, 722]
combos = sorted({(c[3], c[4]) for c in cars})
sizes = list(range(2, 65)) + list(range(68, 129, 4)) + list(range(136, 257, 8)) + list(range(272, 353, 16))
lines, blob = [], bytearray()
for art, body in combos:
    bc = np.array(accel.BODIES[CODES.index(body)], np.float32)
    pic, _ = accel.car_picture(36 * accel.TEX, 14 * accel.TEX, bc, front=art == 1)
    im = Image.fromarray(np.clip(pic, 0, 255).astype(np.uint8), "RGBA")
    for w in sizes:
        h = round(0.875 / 2.25 * (w - 1)) + 1
        small = np.array(im.resize((w, h), Image.BILINEAR)).astype(np.int64)
        # each pixel red, green, blue in tenths of a step and its cover in 32nds, two bytes each (little end first)
        px = np.stack([(small[..., 0] * 310 + 127) // 255, (small[..., 1] * 630 + 127) // 255, (small[..., 2] * 310 + 127) // 255,
                       (small[..., 3] * 32 + 127) // 255], -1).astype("<u2")
        lines.append("%d %d %d %d %d" % (art, body, w, h, len(blob)))
        blob += px.tobytes()
put("traffic.txt", lines)
open(os.path.join(HERE, "traffic.bin"), "wb").write(bytes(blob))
print(len(palette), "colours;", len(lines), "pictures,", len(blob), "bytes")
