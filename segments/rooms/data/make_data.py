"""The rooms' data: the dorm (prototypes/dorm.py) and the screening room (prototypes/screening.py)
as printed tables, from the prototypes' own drawings. Nothing here moves with the frames: the machines light the rooms,
zoom, fade and lay the pictures on each frame.

    python segments/rooms/data/make_data.py          # writes rooms.bin here (not kept in git)

Each room's picture (the dorm whole at 2x, its monitor's surroundings at 8x, the screening room whole at 2x) is a
palette index per art pixel: a pixel's light, as the prototypes light it, is A + B m (per channel: A its own and
the ambient's, B what the screen's mean colour m brings, through the glow, the rim, the beam, the haze); the pairs
(A, B) are put in palettes (each part's square root in steps, the members' mean), so a frame lights a palette, not
the pixels. A and B in ten-thousandths. The screens' masks (where the picture shows) one or none a pixel.

The screens' boxes and middles and the dorm's inset in hundredths of a frame pixel. Tables: lit (ten-thousandths) to sRGB bytes, bytes to linear (thousandths), the eases (thousandths), each zoom's
scale (thousandths) by how far it has gone (thousandths); THE END's cover (0..255) on the 1536 x 768 frame; the
desktop's base (640 x 480 RGB) and the VGA 8x16 face's glyphs (CP437, 256 cells of 16 rows of 8 bits).

The file: records of a name, its dimensions and its numbers (little-endian u32 lengths, i32 values).
"""
import math
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "prototypes"))
sys.path.insert(0, os.path.join(ROOT, "segments", "shared"))
import records    # noqa: E402
import dorm      # noqa: E402
import screening  # noqa: E402

OUT = os.path.join(HERE, "rooms.bin")
REC = []
QA, QB = 256, 384          # the palettes' steps (square roots of A over 1, of B over 4)


def put(name, a):
    a = np.asarray(a)
    assert np.all(np.abs(a) < 2 ** 31), name
    REC.append((name, a.shape, a.astype("<i4")))


def lin(x):
    return np.where(x <= 0.04045, x / 12.92, ((x + 0.055) / 1.055) ** 2.4)


def kmeans(x, w, k, iters=12, seed=1):
    """Weighted k-means of the rows of x (weights w) to k centres; each row's centre."""
    rng = np.random.default_rng(seed)
    c = x[rng.choice(len(x), k, replace=False, p=w / w.sum())].astype(np.float32)
    x32 = x.astype(np.float32)
    xx = (x32 * x32).sum(1)
    for _ in range(iters):
        lab = np.empty(len(x), np.int64)
        cc = (c * c).sum(1)
        for i in range(0, len(x), 8192):
            dist = xx[i:i + 8192, None] - 2 * x32[i:i + 8192] @ c.T + cc[None, :]
            lab[i:i + 8192] = dist.argmin(1)
        tot = np.bincount(lab, weights=w, minlength=k)
        for j in range(x.shape[1]):
            s_ = np.bincount(lab, weights=w * x[:, j], minlength=k)
            c[:, j] = np.where(tot > 0, s_ / np.maximum(tot, 1e-12), c[:, j])
    return lab


def palette(name, L, amb, gain, rim, beam=0.0, haze=0.0, cap=None):
    """A layer set's palette: each pixel's (A, B) per channel, quantised, the palette the members' means; at most
    `cap` entries (the frames light every entry: a rendered picture's many shades are clustered down to it)."""
    la, le = lin(L["alb"]), lin(L["emi"])
    A = la * amb + le * 0.9
    extra = L["rim"] * rim
    if "beam" in L:
        extra = extra + L["beam"] * beam + L["haze"] * haze * 0.05
    B = la * L["glow"][..., None] * gain + extra[..., None]
    h, w = A.shape[:2]
    ka = np.round(np.sqrt(np.clip(A, 0, None)) * QA).astype(np.int64)
    kb = np.round(np.sqrt(np.clip(B, 0, None) / 4.0) * QB).astype(np.int64)
    key = np.concatenate([ka, kb], -1).reshape(-1, 6)
    ukey, inv, cnt = np.unique(key, axis=0, return_inverse=True, return_counts=True)
    inv = inv.reshape(-1)
    if cap and len(cnt) > cap:
        lab = kmeans(ukey / np.array([QA] * 3 + [QB] * 3, float), cnt.astype(float), cap)
        inv = lab[inv]
        _, inv, cnt = np.unique(inv, return_inverse=True, return_counts=True)
        inv = inv.reshape(-1)
    P = len(cnt)
    pal = np.zeros((P, 6))
    vals = np.concatenate([A, B], -1).reshape(-1, 6)
    for c in range(6):
        pal[:, c] = np.bincount(inv, weights=vals[:, c], minlength=P) / cnt
    put(name + "_idx", inv.reshape(h, w))
    put(name + "_pal", np.round(pal * 10000))
    put(name + "_mask", (L["screen"] > 0.5).astype(int))
    print(f"{name}: {w} x {h} art pixels, a palette of {P}; A to {pal[:, :3].max():.3f}, B to {pal[:, 3:].max():.3f}",
          file=sys.stderr)


def hund(box):
    """A box (final-frame units) in hundredths; and its middle."""
    return [round(v * 100) for v in box]


def mid(box):
    return [round((box[0] + box[2]) * 50), round((box[1] + box[3]) * 50)]


def main():
    # the dorm: whole at 2x, the monitor's surroundings at 8x
    Ld = dorm.draw_art(dorm.ART, (0, 0, dorm.W, dorm.H))
    Li = dorm.draw_art(dorm.ART_IN, dorm.INSET)
    palette("dorm", Ld, dorm.AMB, dorm.GAIN, dorm.RIM, cap=3876)        # the flat drawing's sizes: no dearer a frame
    palette("dormin", Li, dorm.AMB, dorm.GAIN, dorm.RIM, cap=1401)
    put("dorm_screen", hund(dorm.SCREEN))
    put("dorm_mid", mid(dorm.SCREEN))
    put("dorm_inset", hund(dorm.INSET))
    # the screening room: whole at 2x
    Ls = screening.draw_art()
    palette("hall", Ls, screening.AMB, screening.GAIN, screening.RIM, screening.BEAM, screening.HAZE, cap=2137)
    put("hall_screen", hund(screening.SCREEN))
    put("hall_mid", mid(screening.SCREEN))
    # lit (ten-thousandths, 0..TOP) to sRGB bytes; bytes to linear thousandths
    TOP = 40000
    x = np.arange(TOP + 1) / 10000
    s = np.where(x <= 0.0031308, x * 12.92, 1.055 * np.clip(x, 0, 1) ** (1 / 2.4) - 0.055)
    put("srgb", np.round(np.clip(s, 0, 1) * 255))
    put("lin8", np.round(lin(np.arange(256) / 255) * 1000))
    # the eases: smoothstep, and smoothstep to the power 1.5 (the screening room's slow start in)
    u = np.arange(1001) / 1000
    e = u * u * (3 - 2 * u)
    put("ease", np.round(e * 1000))
    put("easep", np.round(e ** 1.5 * 1000))
    # each zoom's scale by how far it has gone: s0^(1 - u), thousandths
    dsh, dsw = dorm.SCREEN[3] - dorm.SCREEN[1], dorm.SCREEN[2] - dorm.SCREEN[0]
    for name, s0 in (("dorm_s0", 768 / dsh), ("dorm_sd", 1536 / dsw),
                     ("hall_s0", 1536 / (screening.SCREEN[2] - screening.SCREEN[0]))):
        put(name, np.round(s0 ** (1 - u) * 1000))
    # THE END's cover on the frame
    im = Image.new("L", (1536, 768), 0)
    d = ImageDraw.Draw(im)
    fnt = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 150)
    w = d.textlength("THE END", font=fnt)
    d.text(((1536 - w) / 2, 290), "THE END", font=fnt, fill=255)
    put("end_cover", np.asarray(im))
    # the desktop's base: the prototype's desktop with nothing typed, its text area cleared
    base = screening.desktop_base()
    put("desk_base", base)
    # the VGA 8x16 face's glyphs, CP437 (each cell 16 rows of 8 pixels, one or none)
    f = ImageFont.truetype(screening.VGA, 16)
    cells = np.zeros((256, 16, 8), int)
    for code in range(32, 256):
        ch = bytes([code]).decode("cp437")
        g = Image.new("L", (8, 16), 0)
        ImageDraw.Draw(g).text((0, 0), ch, font=f, fill=255)
        cells[code] = np.asarray(g) > 127
    put("glyphs", cells.reshape(256, 128))
    with open(OUT, "wb") as fo:
        for name, shape, a in REC:
            fo.write(records.record(name, a, shape))
    print(f"{OUT}: {os.path.getsize(OUT) / 1e6:.1f} MB", file=sys.stderr)


if __name__ == "__main__":
    main()
