"""The ending's SQ (the design in prototypes/ending_sq.py): the data CARDIAC works
the frames out from. Nothing here moves: the machines light every pixel and bring the name in.

    python segments/sqend/data/make_data.py          # writes sqend.bin here (not kept in git)

Each pixel of the 1536 x 768 frame as the chrome's surround at an argument T (thousandths: -1000 straight up to 1000
straight down), times M (thousandths), plus A (grey levels): the 3D letters drawn once in perspective by the
prototype (the faces reflecting the surround by their row, the sides by their normal, their bright edges, the seam),
the stars around them (M 0, A their light). The name, STRANGE QUERY: its cover (0..255), its chrome's argument T
(thousandths) and each pixel's turn to come in (0..999).

The file: records of a name, its dimensions and its numbers (little-endian u32 lengths, i32 values), as the rooms'.
"""
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "prototypes"))
sys.path.insert(0, os.path.join(ROOT, "segments", "shared"))
import records  # noqa: E402
import ending_sq as E  # noqa: E402

OUT = os.path.join(HERE, "sqend.bin")


def main():
    T, M, A = E.sq_tma()
    cover, _, order = E.name_layer()
    # the name's argument, as name_layer works it out (by the row over its letters' height)
    rows = np.nonzero(cover.max(1) > 0.5)[0]
    yy = np.mgrid[0:E.H, 0:E.W][0].astype(float)
    tn = np.clip((yy - rows.min()) / max(1, rows.max() - rows.min()) * 2 - 1 + 0.16, -1, 1)
    rec = [("t", np.round(T * 1000)), ("m", np.round(M * 1000)), ("a", np.round(np.clip(A, 0, 255))),
           ("name_cover", np.round(cover * 255)), ("name_t", np.round(tn * 1000)), ("name_order", np.floor(order * 1000).clip(0, 999))]
    with open(OUT, "wb") as fo:
        for name, a in rec:
            fo.write(records.record(name, np.asarray(a).astype("<i4")))
    print(f"{OUT}: {os.path.getsize(OUT) / 1e6:.1f} MB; {int((M > 0).sum())} chrome pixels, {int((cover > 0).sum())} of the name's",
          file=sys.stderr)


if __name__ == "__main__":
    main()
