#!/usr/bin/env python3
"""The turn's data, printed once from the prototype (prototypes/macpc.py): input for the machines and the reference.

    python make_data.py        (with numpy and PIL) writes beside it:

- cga.txt: the CGA tunnel's tables. `bayer`: the 4x4 ordered dither's order (rows of four, 0..15). `hi` and `lo`: for
  each place in the dither (0..15), the squared distance from the middle (in tenths of a pixel, the rows 1.2 times
  as tall, as the prototype's) past which the pixel is a check (hi) or colour 1 (lo): the prototype's level,
  2 ((r - 6) / 110)^1.2, past 1 + t or t for t = (b + 0.5) / 16, as r^2 past a square, rounded down. `fade`: for
  each place in the dither, 6 b + 3, the fade's threshold (the fade's step is 8 a frame). `across` and `down`: each
  column's and each row's place from the middle in tenths of a pixel (the rows 1.2 times as tall), 10 x - 1595 for
  x = 0..159 and 12 y - 1194 for y = 0..99. `pal` lines: the CGA's palettes (black, then the palette's three colours),
  and `flicks`, the palette each second shows.
- mac.txt: the cold open's sine table (segments/mac's, 100 sin(0.075 i), rounded), run on to i = 144: the Mac's
  tunnel runs on under the turn's first 39 frames, to its picture 48. `pictures`: the tunnel's picture at each of
  those frames f, its frame 108 + f, a picture every third frame: (108 + f) / 3.
- grid.txt: the places of the screen's grid's points the rays go through, in tenths of a pixel from the frame's
  middle: `across`, 200 i - 7675 for the grid's columns i = 0..77 (the middles of the pixels 20 i - 768), and `up`,
  3835 - 200 j for its rows j = 0..39 (the middles of the rows 20 j - 384, up); and for each band between two of
  its rows, j = 0..38, its first row on the frame, `top` (20 j), and how many rows it has, `rows` (20, the last 8).
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PROTO = os.path.join(HERE, "../../../prototypes")
sys.path.insert(0, PROTO)


def thresholds():
    bayer = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]
    hi, lo = [], []
    for b in range(16):
        t = (b + 0.5) / 16
        for level, out in ((1 + t, hi), (t, lo)):
            r = 6 + 110 * (level / 2) ** (1 / 1.2)
            out.append(int((10 * r) ** 2))
    return bayer, hi, lo


def main():
    os.chdir(PROTO)                                       # the prototype reads its fonts and face.rgb from there
    import macpc as P
    bayer, hi, lo = thresholds()
    lines = ["# the turn's CGA tunnel: see make_data.py"]
    lines.append("bayer " + " ".join(str(v) for row in bayer for v in row))
    lines.append("hi " + " ".join(map(str, hi)))
    lines.append("lo " + " ".join(map(str, lo)))
    lines.append("fade " + " ".join(str(6 * b + 3) for b in range(16)))
    lines.append("across " + " ".join(str(10 * x - 1595) for x in range(160)))
    lines.append("down " + " ".join(str(12 * y - 1194) for y in range(100)))
    names = ["pal0 high", "pal1 high", "mode5 high", "pal0", "pal1"]
    for n in names:
        lines.append("pal " + " ".join(str(v) for c in P.CGA[n] for v in c) + "  # " + n)
    lines.append("flicks " + " ".join(str(names.index(n)) for n in P.FLICKS))
    with open(os.path.join(HERE, "cga.txt"), "w") as f:
        f.write("\n".join(lines) + "\n")
    with open(os.path.join(HERE, "mac.txt"), "w") as f:
        f.write("# the cold open's sine table run on: see make_data.py\n")
        f.write("sine " + " ".join(str(round(100 * math.sin(0.075 * i))) for i in range(145)) + "\n")
        f.write("pictures " + " ".join(str((108 + f) // 3) for f in range(39)) + "\n")
    with open(os.path.join(HERE, "grid.txt"), "w") as f:
        f.write("# the turn's screen grid: see make_data.py\n")
        f.write("across " + " ".join(str(200 * i - 7675) for i in range(78)) + "\n")
        f.write("up " + " ".join(str(3835 - 200 * j) for j in range(40)) + "\n")
        f.write("top " + " ".join(str(20 * j) for j in range(39)) + "\n")
        f.write("rows " + " ".join(str(min(20, 768 - 20 * j)) for j in range(39)) + "\n")

if __name__ == "__main__":
    main()
