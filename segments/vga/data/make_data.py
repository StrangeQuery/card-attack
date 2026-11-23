#!/usr/bin/env python3
"""The VGA segment's data, made once: the EGA's sixteen colours in index order (blue, green and red bits, then the
intensity bit; colour 6 the brown, not dark yellow), as paint16's palette; the reference reads them too.

    python make_data.py       writes ega.txt, ansi.txt and wipe.txt beside it

ansi.txt: the 16-colour plasma's code by the plasma's place on its colour ring (as the machines leave it, 0..511: the
waves and the colours' start added, before mod 256): that mod 256, its sixteenth, so the
sixteen colours go round in their order as a 16-colour plasma's palette did. wipe.txt: the
switch to 256 colours, a frame at a time: the first row still in 16 colours (its line white), 16 rows a frame.
"""
import os

EGA = [(0, 0, 0), (0, 0, 170), (0, 170, 0), (0, 170, 170), (170, 0, 0), (170, 0, 170), (170, 85, 0), (170, 170, 170),
       (85, 85, 85), (85, 85, 255), (85, 255, 85), (85, 255, 255), (255, 85, 85), (255, 85, 255), (255, 255, 85),
       (255, 255, 255)]

here = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(here, "ega.txt"), "w") as f:
    f.write("# the EGA's sixteen colours in index order, red green blue: see make_data.py\n")
    for c in EGA:
        f.write(" ".join(map(str, c)) + "\n")

with open(os.path.join(here, "ansi.txt"), "w") as f:
    f.write("# the 16-colour plasma's code by its place on the colour ring 0..511 (mod 256): see make_data.py\n")
    f.write(" ".join(str(q % 256 // 16) for q in range(512)) + "\n")
with open(os.path.join(here, "wipe.txt"), "w") as f:
    f.write("# the wipe to 256 colours: each frame's first row in 16 colours (white), see make_data.py\n")
    f.write(" ".join(str(16 * (k + 1)) for k in range(6)) + "\n")
