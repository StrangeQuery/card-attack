#!/usr/bin/env python3
"""The degauss's tables, made once (prototypes/degauss.py's curves, sampled); the machines and the reference read them.

    python make_data.py       writes gauss.txt, decay.txt, atan.txt, rates.txt, cues.txt and sign.txt beside it

gauss.txt: the two colour blotches' falloffs, per 320x240 column then per row, at the middle of the pixel's 3.2 x 3.2
    on the 1536x768 frame ((256 + 3.2 u + 1.6, 3.2 v + 1.6)): 32 exp(-((x - 330) / 420)^2) rounded, at most 31; the
    first blotch's across (320), the second's across (320), the first's down (240), the second's down (240).
decay.txt: the jolt's decay, 999 exp(-j / 4) rounded (at most 999), j = 0..15; then the flash's, 999 exp(-j / 2).
atan.txt: 10 atan(k / 256) in degrees, rounded, k = 0..256 (0..450).
rates.txt: how fast the waves' angles turn, as the clock machine (dr_clock) takes a rate: quarter turns, tenths of
    a degree (0..899), thousandths of a tenth, rounded: 1 / 37 and 1 / 11 radians a row, 1 / 90 a column; 2.1, -3.57
    (-1.7 x 2.1, as the turn round it) and 2.73 (1.3 x 2.1) radians a frame.
cues.txt: the sound cues, for when the film plays sound: frame of Mode X (12 a second), seconds, what.
sign.txt: Mode X's sign (VGA over MODE X, liquid chrome behind the ball; ui/pages.rs's MODEX_SIGN, see reference.rs):
    a row per lettered pixel of the 320x240 picture, row by row: x, y, its look as the plasma's tone takes it
    (100 + 32 dark + fog) and its point on the plasma, u and v, its 2 x 2 block's (the block's top left pixel's).
"""
import math, os

here = os.path.dirname(os.path.abspath(__file__))


def falloff(at, mid, size):
    return min(31, int(math.floor(32 * math.exp(-((at - mid) / size) ** 2) + 0.5)))


def write(name, head, rows):
    with open(os.path.join(here, name), "w") as f:
        f.write(f"# {head}: see make_data.py\n")
        for r in rows:
            f.write(" ".join(map(str, r)) + "\n")


xs = [256 + 3.2 * u + 1.6 for u in range(320)]
ys = [3.2 * v + 1.6 for v in range(240)]
write("gauss.txt", "the blotches' falloffs in 32nds: first across, second across, first down, second down",
      [[falloff(x, 330, 420) for x in xs], [falloff(x, 1250, 360) for x in xs],
       [falloff(y, 560, 300) for y in ys], [falloff(y, 170, 260) for y in ys]])
# the same blotches over the rotozoom's 192x96 picture filling the frame (rotodg: the degauss moved to
# the rotozoom's end from Mode X's), at the middle of each pixel's 8 x 8
write("gauss_roto.txt", "the blotches' falloffs over 192x96 in 32nds: first across, second across, first down, second down",
      [[falloff(8 * u + 4, 330, 420) for u in range(192)], [falloff(8 * u + 4, 1250, 360) for u in range(192)],
       [falloff(8 * v + 4, 560, 300) for v in range(96)], [falloff(8 * v + 4, 170, 260) for v in range(96)]])
write("decay.txt", "999 exp(-j / 4), then 999 exp(-j / 2), j = 0..15",
      [[min(999, int(math.floor(999 * math.exp(-j / 4) + 0.5))) for j in range(16)],
       [min(999, int(math.floor(999 * math.exp(-j / 2) + 0.5))) for j in range(16)]])
write("atan.txt", "10 atan(k / 256) degrees, k = 0..256", [[int(math.floor(10 * math.degrees(math.atan(k / 256)) + 0.5)) for k in range(257)]])


def rate(radians):
    t = math.degrees(radians) * 10 % 3600                 # tenths of a degree, round the turn
    whole = int(t)
    th = int(math.floor((t - whole) * 1000 + 0.5))
    if th == 1000: whole, th = whole + 1, 0
    return [whole // 900, whole % 900, th]


write("rates.txt", "the waves' rates: quarter turns, tenths, thousandths; 1/37, 1/11, 1/90 rad a row or column, "
      "2.1, -3.57, 2.73 rad a frame", [rate(1 / 37), rate(1 / 11), rate(1 / 90), rate(2.1), rate(-3.57), rate(2.73)])

with open(os.path.join(here, "cues.txt"), "w") as f:
    f.write("# the sound cues: frame of Mode X (12 a second), seconds, what: see make_data.py\n")
    f.write(f"134 {134 / 12:.2f} the degauss: a thunk as the picture jolts, then a hum dying away with the wobble (16 frames, to 149)\n")


# the sign: ui/pages.rs's MODEX_SIGN drawn from the atlas's VGA 8x16 face (11): VGA centred on 24, MODE X from 0, a
# line every 12 page pixels from -2, each page pixel 6 x 4 on the picture at (13, 6); a picture pixel lettered where
# its page pixel has ink (white on the picture; the host outlines it in black on CARDIAC)
glyph = {}
atlas = open(os.path.join(here, "../../ui/data/atlas.bin"), "rb").read()
for l in open(os.path.join(here, "../../ui/data/atlas.txt")):
    p = l.split()
    if p and p[0] == "g" and p[1] == "11":
        glyph[int(p[2])] = (int(p[8]), int(p[6]), int(p[7]))      # its covers' offset, width, height
PW, PH = 49, 22
page = [[False] * PW for _ in range(PH)]
for n, (x0, text) in enumerate(((24 - 3 * 8 // 2, "VGA"), (0, "MODE X"))):
    for k, ch in enumerate(text):
        at, gw, gh = glyph[ord(ch)]
        for r in range(gh):
            for c in range(gw):
                gx, gy = x0 + 8 * k + c, -2 + 12 * n + r
                if 0 <= gx < PW and 0 <= gy < PH and atlas[at + r * gw + c]:
                    page[gy][gx] = True


def inked(x, y):
    if x < 13 or y < 6:
        return False
    u, v = (x - 13) // 6, (y - 6) // 4
    return u < PW and v < PH and page[v][u]


write("sign.txt", "Mode X's sign: per lettered pixel x, y",
      [[x, y] for y in range(240) for x in range(320) if inked(x, y)])
