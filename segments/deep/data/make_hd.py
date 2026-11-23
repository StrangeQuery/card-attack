#!/usr/bin/env python3
"""The dive's HD creatures: input data, made once here, like the fish atlas itself.

    python make_hd.py      writes hd_fish.bin, hd_bell.bin and hd_bubble.bin beside it (built data, not kept: run it
                           after a checkout), and hd.txt (their index and the printed tables; kept)

- hd_fish.bin: the fish atlas (fish_atlas.png, fish_flash.png, made by make_fish_atlas.py) at nine sizes, each cell
  128 x 64 and then smaller by the square root of 2 a step (the cells resampled one at a time, premultiplied, so no
  cell bleeds into the next). A level is its 26 rows (elevation * 13 + yaw) of 8 cells (swim phases), each cell's
  rows of texels: red, green, blue, cover (0..255, straight) and the flash map (0..255).
- hd_bell.bin: the jellyfish bells, ray-cast as the look preview drew them, but
  seen from far away (straight lines of sight) from seven heights (30 degrees below to 60 above, every 15) at
  eight sizes of the pulse; 24 texels a tenth of a unit, the bell's middle at the cell's middle; at thirteen sizes
  smaller by the square root of 2 a step. Each texel: the walls' glow (hundredths, at most 255) and the rim's light
  (hundredths, at most 100). Cell order: pulse * 7 + view.
- hd_bubble.bin: the bubbles (the marine snow made tiny rising bubbles): a thin bright rim, a glint up and to the
  left, a clear middle, as the preview drew them, at 60 sizes (radius 1.1 pixels times 1.08 a step), each in a box
  of side 2 ceil(r) + 5 with the middle at its middle pixel's middle offset by quarters of a pixel (16 offsets for
  the sizes up to 8 pixels, else 1): each texel round(255 min(1, 0.45 rim + fill + 0.6 glint)).
- hd.txt: each file's levels, sizes and offsets, and the tables the machines are dealt.
"""
import math
import os

import numpy as np
from PIL import Image

here = os.path.dirname(os.path.abspath(__file__))

FISH_W = [128, 91, 64, 45, 32, 23, 16, 11, 8]
FISH_H = [w // 2 for w in FISH_W]
YAWS, ELEVS, PHASES = 13, 2, 8

BELL_W = [round(600 * 2 ** (-k / 2)) for k in range(13)]
BELL_TEXELS_PER_TENTH = 24
BELL_VIEWS = [-30, -15, 0, 15, 30, 45, 60]
BELL_PULSES = 8

BUB_R = [1.1 * 1.08 ** i for i in range(60)]


def resample(cell, w, h):
    """A cell (float, premultiplied, channels last) at another size, by Lanczos (Pillow, a channel at a time)."""
    if cell.shape[1] == w and cell.shape[0] == h:
        return cell
    out = np.zeros((h, w, cell.shape[2]))
    for c in range(cell.shape[2]):
        im = Image.fromarray(cell[..., c].astype(np.float32), mode="F")
        out[..., c] = np.asarray(im.resize((w, h), Image.LANCZOS))
    return out


def fish():
    rgba = np.asarray(Image.open(os.path.join(here, "fish_atlas.png")).convert("RGBA")).astype(np.float64) / 255
    flash = np.asarray(Image.open(os.path.join(here, "fish_flash.png")).convert("L")).astype(np.float64) / 255
    a = rgba[..., 3:4]
    pm = np.concatenate([rgba[..., :3] * a, a, flash[..., None] * a], -1)     # premultiplied, the flash too
    blob = bytearray()
    offsets = []
    for w, h in zip(FISH_W, FISH_H):
        offsets.append(len(blob))
        for row in range(YAWS * ELEVS):
            cells = []
            for ph in range(PHASES):
                c = resample(pm[row * 64:(row + 1) * 64, ph * 128:(ph + 1) * 128], w, h)
                al = np.clip(c[..., 3:4], 0, 1)
                straight = np.where(al > 1e-4, c / np.maximum(al, 1e-4), 0)
                t = np.concatenate([straight[..., :3], al, straight[..., 4:5]], -1)
                t[al[..., 0] < 0.5 / 255] = 0
                cells.append(np.clip(np.round(t * 255), 0, 255).astype(np.uint8))
            for c in cells:
                blob += c.tobytes()
    open(os.path.join(here, "hd_fish.bin"), "wb").write(bytes(blob))
    return offsets


def bell_cell(elev, pulse, size, ss=2):
    """The bell seen from elev degrees above, straight lines of sight: (glow, rim) per texel, as the preview's
    jelly_layer works them out (its turn 0)."""
    rad, ht = pulse, 0.75 / pulse            # units: tenths of a unit (the preview's rad / 10, ht / 10)
    rad, ht = 10 * rad, 10 * ht
    e = math.radians(elev)
    d = np.array([0.0, -math.sin(e), math.cos(e)])           # the line of sight, down when seen from above
    right = np.array([1.0, 0.0, 0.0])
    up = np.array([0.0, math.cos(e), math.sin(e)])
    n = size * ss
    g = (np.arange(n) + 0.5) / ss - size / 2                 # texels from the middle
    gx, gy = np.meshgrid(g / BELL_TEXELS_PER_TENTH, -g / BELL_TEXELS_PER_TENTH)
    o = gx[..., None] * right + gy[..., None] * up - 40 * d  # each line's start, from the bell's middle
    sc = np.array([1 / rad, 1 / ht, 1 / rad])
    oo = o * sc
    dd = d * sc
    A = np.sum(dd * dd)
    B = 2 * np.sum(oo * dd, -1)
    C = np.sum(oo * oo, -1) - 1
    disc = B * B - 4 * A * C
    hit = disc > 0
    sq = np.sqrt(np.maximum(disc, 0))
    bell = np.zeros(gx.shape)
    rimg = np.zeros(gx.shape)
    for k, tt in enumerate(((-B - sq) / (2 * A), (-B + sq) / (2 * A))):
        p = o + d * tt[..., None]
        phi = np.arctan2(p[..., 2], p[..., 0])
        rim = -0.10 * ht + 0.05 * ht * np.cos(16 * phi)
        on = hit & (p[..., 1] > rim)
        nn = p * sc * sc
        nn /= np.linalg.norm(nn, axis=-1, keepdims=True) + 1e-9
        side_on = 1 - np.abs(np.sum(nn * d, -1))
        rho = np.hypot(p[..., 0], p[..., 2]) / rad
        canals = 1 + 0.6 * np.maximum(np.cos(8 * phi), 0) ** 16 * rho
        gon = 0.55 * np.exp(-((rho - 0.38) / 0.10) ** 2) * np.maximum(np.cos(4 * phi), 0) ** 2 * (k == 0)
        wall = (0.22 + 1.15 * side_on ** 2.0) * canals + gon
        wall = wall * (1.0 if k == 0 else 0.55)
        bell += np.where(on, wall, 0)
        rimg += np.where(on, np.exp(-((p[..., 1] - rim) / (0.07 * ht)) ** 2), 0)
    bell = bell.reshape(size, ss, size, ss).mean((1, 3))
    rimg = rimg.reshape(size, ss, size, ss).mean((1, 3))
    edge = 0.9 * rimg + 0.6 * np.clip(bell - 0.5, 0, 1)
    return np.stack([bell, edge], -1)


def bells():
    blob = bytearray()
    offsets = []
    cells = []
    for p in range(BELL_PULSES):
        pulse = 0.85 + 0.30 * p / (BELL_PULSES - 1)
        for elev in BELL_VIEWS:
            cells.append(bell_cell(elev, pulse, BELL_W[0]))
    for w in BELL_W:
        offsets.append(len(blob))
        for c in cells:
            r = resample(c, w, w)
            glow = np.clip(np.round(100 * r[..., 0]), 0, 255)
            edge = np.clip(np.round(100 * r[..., 1]), 0, 100)
            blob += np.stack([glow, edge], -1).astype(np.uint8).tobytes()
    open(os.path.join(here, "hd_bell.bin"), "wb").write(bytes(blob))
    return offsets


def bubbles():
    blob = bytearray()
    index = []
    for R in BUB_R:
        side = 2 * math.ceil(R) + 5
        phases = 4 if R <= 8 else 1
        index.append((len(blob), side, phases))
        h = side // 2
        for sy in range(phases):
            for sx in range(phases):
                # the middle: the box's middle pixel's corner, plus (s + 0.5) quarters (a half for one phase)
                cx = h + ((sx + 0.5) / 4 if phases == 4 else 0.5)
                cy = h + ((sy + 0.5) / 4 if phases == 4 else 0.5)
                yy, xx = np.mgrid[0:side, 0:side].astype(float) + 0.5
                d = np.hypot(xx - cx, yy - cy)
                w = max(0.55, 0.22 * R)
                rim = np.exp(-((d - R) / w) ** 2)
                fill = 0.04 * (d < R)
                glint = np.exp(-(np.hypot(xx - (cx - 0.4 * R), yy - (cy - 0.4 * R)) / max(0.5, 0.22 * R)) ** 2)
                raw = 0.45 * rim + fill + 0.6 * glint
                blob += np.clip(np.round(255 * np.minimum(raw, 1)), 0, 255).astype(np.uint8).tobytes()
    open(os.path.join(here, "hd_bubble.bin"), "wb").write(bytes(blob))
    return index


def main():
    fo = fish()
    bo = bells()
    bi = bubbles()
    L = []
    L.append("# the dive's HD creatures: see make_hd.py")
    L.append("fish_w " + " ".join(map(str, FISH_W)))
    L.append("fish_h " + " ".join(map(str, FISH_H)))
    L.append("fish_at " + " ".join(map(str, fo)))
    L.append("bell_w " + " ".join(map(str, BELL_W)))
    L.append("bell_at " + " ".join(map(str, bo)))
    # the sizes as the machines take them: 7 and 500 times a fish's width, 14 and 500 times its height (the level
    # test and the texel steps, the walk's middle texel in thousandths), 500 times a bell's
    L.append("fish_w7 " + " ".join(str(7 * w) for w in FISH_W))
    L.append("fish_w500 " + " ".join(str(500 * w) for w in FISH_W))
    L.append("fish_h14 " + " ".join(str(14 * h) for h in FISH_H))
    L.append("fish_h500 " + " ".join(str(500 * h) for h in FISH_H))
    L.append("bell_w500 " + " ".join(str(500 * w) for w in BELL_W))
    L.append("bubble_at " + " ".join(str(o) for o, _, _ in bi))
    L.append("bubble_side " + " ".join(str(s) for _, s, _ in bi))
    L.append("bubble_phases " + " ".join(str(p) for _, _, p in bi))
    # a bubble's size by how far it is ahead (tenths, 0..399): the nearest of the sizes to 280 / ahead pixels
    # (the preview's 800 0.35 / ahead), at least 1.1
    size = []
    for a in range(400):
        R = max(1.1, 280 / a) if a > 0 else BUB_R[-1]
        size.append(min(range(len(BUB_R)), key=lambda i: abs(math.log(BUB_R[i] / R))))
    L.append("bubble_size " + " ".join(map(str, size)))
    # the jellyfish's cover by its glow e (hundredths): 64 (1 - exp(-2.2 e))
    L.append("cover " + " ".join(str(round(64 * (1 - math.exp(-2.2 * e / 100)))) for e in range(300)))
    # toward the pale edge colour (32nds) by the glow and by the rim: 0.35 e and 0.6 rim
    L.append("pale_glow " + " ".join(str(min(32, round(32 * 0.35 * e / 100))) for e in range(300)))
    L.append("pale_rim " + " ".join(str(round(32 * 0.6 * r / 100)) for r in range(101)))
    # a strand's piece's glow (hundredths): across it 100 exp(-x^2 / 2), x eighths of its width (0..23, either side);
    # along it, from half a piece before it to half a piece after (hundredths of the piece, -50..149), up from
    # nothing and down again, so that each piece's glow and the next's add up to the whole along the strand
    hat = [(l + 50 + 0.5) / 100 if l < 50 else (150 - l - 0.5) / 100 for l in range(-50, 150)]
    L.append("strand " + " ".join(str(round(100 * math.exp(-((k + 0.5) / 8) ** 2 / 2) * h)) for k in range(24) for h in hat))
    # the halo: 56 exp(-k / 100), k half the squared distance in tenths of its width (0..961), so that a fade f
    # (64ths) mixes f 56 exp(-k / 100) / 40 256ths: 0.35 exp(-r^2 / 2 w^2) f / 64
    L.append("halo " + " ".join(str(round(56 * math.exp(-k / 100))) for k in range(962)))
    # the fine tentacles' lengths: 100 (0.75 + 0.2 sin tn), tn = 0..23 (the long ones, every third, 100)
    L.append("fine " + " ".join(str(100 if tn % 3 == 0 else round(100 * (0.75 + 0.2 * math.sin(tn)))) for tn in range(24)))
    # a strand's strength along it, 100 (1 - a)^1.3, a the middle of each of its 24 pieces
    L.append("taper " + " ".join(str(round(100 * (1 - (k + 0.5) / 24) ** 1.3)) for k in range(24)))
    L.append("arm_taper " + " ".join(str(round(100 * (1 - (k + 0.5) / 20))) for k in range(20)))
    # the orcas' haze by how far ahead they are (whole units, 0..199): the dive's fog, 32 min(1, 0.85 (a / 70)^0.8)
    L.append("orca_haze " + " ".join(str(round(32 * min(1.0, 0.85 * (a / 70) ** 0.8))) for a in range(200)))
    open(os.path.join(here, "hd.txt"), "w").write("\n".join(L) + "\n")


if __name__ == "__main__":
    main()
