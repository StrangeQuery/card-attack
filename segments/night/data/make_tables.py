"""The night arrival's printed tables (beside the train's, segments/train/data/tables.bin): the curves the night's
machines look things up in by the indices they punch (the host fetches; it works nothing out), each a function of
the prototype's (prototypes/train_night.py) tabulated in whole numbers.

    python segments/night/data/make_tables.py        # writes segments/night/data/tables.bin (not kept in git)
"""
import importlib.util
import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..", "..")
sys.path.insert(0, os.path.join(ROOT, "prototypes"))
sys.path.insert(0, os.path.join(ROOT, "segments", "shared"))
import records                                            # noqa: E402
spec = importlib.util.spec_from_file_location("train_tables", os.path.join(ROOT, "segments", "train", "data", "make_tables.py"))
TT = importlib.util.module_from_spec(spec)
spec.loader.exec_module(TT)                                # the train's: fbm, fstripe, fw_cm, LEVELS
import train_night as N                                    # noqa: E402

OUT = os.path.join(HERE, "tables.bin")
ARR = []


def put(name, a):
    a = np.ascontiguousarray(np.round(np.asarray(a, float)), dtype="<i8")
    assert np.abs(a).max() < 2 ** 31, name
    ARR.append((name, a.astype("<i4")))


def main():
    x = np.arange(20_000)
    put("expn", np.exp(-x / 1000) * 1000)
    x = np.arange(1001) / 1000
    put("pow115", x ** 1.15 * 1000)
    put("pow22", x ** 2.2 * 1000)
    put("pow055", x ** 0.55 * 1000)
    put("sqrtk", np.sqrt(x) * 1000)
    put("pow15", x ** 1.5 * 1000)
    # the speed's Gaussian (4 frames, 25 taps), thousandths summing to 1000
    k = np.arange(-12, 13)
    w = np.exp(-0.5 * (k / 4.0) ** 2)
    w = np.round(w / w.sum() * 1000)
    w[12] += 1000 - w.sum()
    put("gauss4", w)
    r = np.arange(-10_000, 10_001)
    put("atan4", np.arctan(r / 10_000) * 10_000)
    xx = np.arange(-5000, 5001)
    put("atan_x", np.arctan(xx / 1000) * 1000)
    i = np.arange(4001)
    put("sq2000", np.minimum((i - 2000) ** 2, 999_999))
    i = np.arange(19_999)
    put("sq9999", (i - 9999) ** 2)
    # the sky going into night, by the way's height part (-1000..1000)
    el = np.arange(-1000, 1001) / 1000
    put("tw_shape", (1 - np.clip(el / 0.6, 0, 1)) * 1000)
    put("glow_shape", np.exp(-np.maximum(el, 0) / 0.04) * 1000)
    put("hz_shape", np.exp(-np.maximum(el, 0) * 9) * 1000)
    # the stadium's glow by the squared distance of the way from its way over 100 (millionths): 1 - cos = q 5e-5
    q = np.arange(30_000) * 5e-5
    put("sg_shape", (0.35 * np.exp(-q / 0.004) + 0.08 * np.exp(-q / 0.03)) * 1000)
    # the screen's numbers by q = 1000 f2 / |ray| (half pixels): as the train's scr_nlq7 and scr_lq
    qq = np.arange(1000)
    q7 = np.maximum(np.floor(qq * 1000 / 1400), 1)
    put("scr_nlq7_q", np.floor(np.log(500 / q7) * 10000))
    qf = np.floor(700_000 / np.maximum(qq, 1))
    put("scr_lq_q", np.floor(np.log(qf) * 1000) + 6059 + 3000)
    # the board's dots (0 dark, 1 orange, 2 amber), edged with dark ones; a lit dot's glow by the footprint's level
    put("nled_bits", np.pad(N.LED.astype(int), 1))
    L = np.arange(TT.LEVELS)
    put("nled_fill", 100 * 0.5 * np.clip(TT.fw_cm(L) / 7.25 - 0.5, 0, 1) / 9)
    # the airship's plates' seams: stripes every 3.1 m along and 2.3 m round, 3% wide, by the footprint's level
    for name, period in (("bh_s31", 1e5 / 323), ("bh_s23", 1e5 / 435)):
        u = (np.arange(1000) + 0.5) / 1000 * period
        Lc = np.arange(TT.LEVELS)[:, None]
        put(name, np.round(TT.fstripe(u[None, :], period, 0.03, TT.fw_cm(Lc)) * 100).reshape(TT.LEVELS, 1000))
    # its grime: fbm of 3 octaves over a 100 m tile (15 by 20 cells), decimetres
    put("n_bhull", np.clip(TT.fbm(1000, 15, 20, 3, 131), 0, 0.999) * 1000)
    # logarithms and back (ten-thousandths): ln x for 1..999,999; e^(k / 10,000) for k -50,000..115,999 (R hundredths)
    n = np.arange(1_000_000)
    put("ln6", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 10000), -500_000))
    k = np.arange(-50_000, 116_000)
    put("exp6", np.minimum(np.round(np.exp(k / 10_000)), 999_999))
    # the lamps' distance functions (ten-thousandths of ln, d in metres): by the squared distance in dm^2 (to 100 m)
    # and m^2 beyond: the tubes' d (d + 1.5), the street lights' d (d^2 + 4), the headlights' d (d^2 + 1)
    da = np.sqrt(np.maximum(n, 0.25)) / 10
    db = np.sqrt(np.maximum(n, 0.25))
    for name, fn in (("lnL1", lambda d: d * (d + 1.5)), ("lnL2", lambda d: d * (d * d + 4)), ("lnL3", lambda d: d * (d * d + 1))):
        put(name + "a", np.floor(np.log(fn(da)) * 10000))
        put(name + "b", np.floor(np.log(fn(db)) * 10000))
    # a beam's Gaussian e^-(d / w)^2 as its logarithm (ten-thousandths) by ln (d / w) (ten-thousandths, from -50,000)
    k = np.arange(-50_000, 31_000)
    put("lngauss", np.maximum(-np.exp(2 * k / 10_000) * 10_000, -900_000))
    # the hull: lit from below by -n.y (thousandths, by n.y in hundredths -127..127); a pool by the squared
    # distance from its axis (dm^2): e^-(r^2 / 30 m^2)
    ny = np.arange(-127, 128)
    put("hull_under", np.clip(-ny * 10, 0, 1000))
    r2 = np.arange(200_000)
    put("hull_pool", np.exp(-r2 / 100 / 30) * 1000)
    dz = np.arange(10_000) / 10
    put("lnatt", np.floor(np.log(1 + 0.15 * dz) * 10000))
    put("lnhalf", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 5000), 0))
    w = np.arange(10_000) / 10
    put("ipow03", 1000 / np.maximum(w, 1) ** 0.3)
    c = np.arange(256)
    put("pow22x", (c / 255) ** 2.2 * 2.2 * 1000)
    # the airship's picture (blimp_screen) by its place's parts, a across and b up (thousandths 0..999): what depends
    # on one of them alone, so a pixel only adds and looks up (thousandths of 1.0)
    t = np.arange(1000) / 1000
    hz = 0.62
    above = t > hz
    sky = np.clip((t - hz) / (1 - hz), 0, 1)
    put("bs_up", np.floor(np.array([0.9, 0.08, 0.45])[:, None] * 1.6 * (1 - sky)[None, :] * above[None, :] * 1000))
    # the glow e^-((b - 0.76) / 0.07)^2 e^-((a - 0.55) / 0.3)^2 as e^-(xb + xa): the exponents (thousandths), and the
    # glow's colour by their sum
    put("bs_xb", np.minimum(((t - 0.76) / 0.07) ** 2 * 1000, 19_999))
    put("bs_xa", np.minimum(((t - 0.55) / 0.3) ** 2 * 1000, 19_999))
    s = np.arange(20_000) / 1000
    put("bs_glow", np.floor(np.array([1.4, 1.1, 0.5])[:, None] * 2.0 * np.exp(-s)[None, :] * 1000))
    # the grid below the horizon: its depth Zg (thousandths, to 220), its lines' widths, the cross lines' fade by
    # depth, the fade at the horizon
    zg = 0.22 / np.maximum(hz - t, 1e-3)
    put("bs_zg", np.minimum(np.round(zg * 1000), 220_000))
    put("bs_wz", np.round((0.48 - 0.06 / np.maximum(zg, 0.3)) * 1000))
    put("bs_cx", np.round(np.clip(1.5 / zg, 0, 1) * 1000))
    put("bs_fade", np.round(np.clip((hz - t) / 0.08, 0, 1) * 1000))
    h = np.arange(501) / 1000
    x = np.clip((h - 0.465) / 0.035, 0, 1)
    put("bs_lx", np.round(x * x * (3 - 2 * x) * 1000))
    ln = np.arange(1001) / 1000
    put("bs_line", np.floor((np.array([1.6, 0.75, 0.12])[:, None] * ln[None, :] + np.array([0.12, 0.02, 0.08])[:, None]) * 1000))
    # the words' texel (row, column; and 6 up and left for the shadow), their colour by the coverage (percent)
    put("bs_iy", np.clip(np.floor((1 - t) * 559), 0, 559))
    put("bs_ix", np.clip(np.floor(t * 999), 0, 999))
    put("bs_iy6", np.clip(np.floor((1 - t) * 559) - 6, 0, 559))
    put("bs_ix6", np.clip(np.floor(t * 999) - 6, 0, 999))
    cv = np.arange(101) / 100
    put("bs_word", np.floor(np.array([0.3, 0.75, 2.4])[:, None] * 1.6 * cv[None, :] * 1000))
    # the picture's lines (0.85 + 0.15 sin(560 pi b), thousandths); the knob's picture's row by b
    put("bs_scan", np.round((0.85 + 0.15 * np.sin(t * 560 * np.pi)) * 1000))
    put("div25l", np.minimum(np.arange(25_000) // 25, 999))
    put("mask1000", np.arange(256) * 1000 // 255)
    put("vol_row", np.clip(np.floor((1 - t) * 560), 0, 559))
    blur_half()
    with open(OUT, "wb") as f:
        for name, a in ARR:
            # two dimensions always (the train's Tb): rows and their width (1 for a list)
            rows, w = (a.shape[0], 1) if a.ndim == 1 else a.shape
            f.write(records.record(name, a, (rows, w)))
    for name, a in ARR:
        print(name, a.shape, a.min(), a.max())


def blur_half():
    """The motion blur's lines for the party version's world at 768x384 (the train's blur_tables at half the size):
    for each of 16 directions and each pixel its line and place along it; each line's length."""
    W, H = 768, 384
    ys, xs = np.mgrid[0:H, 0:W]
    lines, places, lens = [], [], []
    for d in range(16):
        a = math.pi * d / 16
        c, sn = math.cos(a), math.sin(a)
        if abs(c) >= abs(sn):
            off = np.round(xs * (sn / c)).astype(int)
            lid, pos = ys - off, xs
        else:
            off = np.round(ys * (c / sn)).astype(int)
            lid, pos = xs - off, ys
        lid = lid - lid.min()
        first = np.full(lid.max() + 1, 10 ** 9)
        np.minimum.at(first, lid.ravel(), pos.ravel())
        last = np.full(lid.max() + 1, -1)
        np.maximum.at(last, lid.ravel(), pos.ravel())
        lines.append(lid.ravel()); places.append((pos - first[lid]).ravel()); lens.append(last - first + 1)
    put("blur_line_h", np.stack(lines))
    put("blur_place_h", np.stack(places))
    m = max(len(l) for l in lens)
    put("blur_len_h", np.stack([np.pad(l, (0, m - len(l))) for l in lens]))


if __name__ == "__main__":
    main()
