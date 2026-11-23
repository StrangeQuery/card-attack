"""The bullet train's printed tables: the curves, noises and filtered patterns the machines look things up in by the
indices they punch (the host fetches; it works nothing out). Each is the prototype's function (prototypes/train.py)
tabulated, in whole numbers.

    python segments/train/data/make_tables.py        # writes segments/train/data/tables.bin (not kept in git)
    python segments/train/data/make_tables.py --half # writes tables_h.bin: the half resolution's blur lines

Noise: the prototype's value-noise fbm, made to repeat (a tile of 10 m, or 10 km for the clouds) so it can be
looked up by a place's last digits; each use its own table with its cells' size baked in. Filtered stripes and bands
(the prototype's fstripe and fband: the share of a pixel's footprint on a stripe) by a footprint's level, quarter
octaves of millimetres.
"""
import math
import os
import sys

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "shared"))
import records                                            # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "tables.bin")
ARR = []


LONGS = {"scr_nlq7", "lnr0", "sq999c", "sq577", "half_ln4", "ln99c4", "lnz_f", "lnz_f2", "lnz_f4", "kz_z10", "kz10_z10", "exps4", "exps4n", "scr_lq", "ln_off4", "lnd4", "x25", "g_hw", "lniz", "ln_foot_ic", "lniz4", "lnr4", "exp4", "exp4n", "ln", "ln_m1", "glow_dec", "lnv", "lnk", "ln_s", "ln_n", "exp_s", "glint0", "glint1", "glint2", "exp_long", "blur_step", "head_g", "cloudk_day", "cloudk_dusk", "recip_ahead", "conc_jl", "map_az", "round_az", "atan", "lev"}


def put(name, a, w=0):
    a = np.asarray(np.round(a))
    if name not in LONGS and not name.startswith(("blur_", "lampgrid_", "headgrid_", "ink_box")):
        a = np.minimum(a, 999)                  # a card: 1.0 in thousandths is held to 999
    a = np.ascontiguousarray(a, dtype="<i4")
    if a.ndim == 2:
        w = a.shape[1]
    ARR.append((name, a.reshape(-1), w))


def smooth(a, b, x):
    t = np.clip((np.asarray(x, float) - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


# ---- periodic value noise ---------------------------------------------------------------------------------------

def pnoise(n, px, py, seed):
    """Value noise over a tile of n x n texels holding px by py cells, repeating: smoothstep between random values at
    the cells' corners. Rows along y, columns along x."""
    rng = np.random.default_rng(seed)
    lat = rng.random((py, px))
    x = (np.arange(n) + 0.5) * px / n
    y = (np.arange(n) + 0.5) * py / n
    xi, yi = np.floor(x).astype(int), np.floor(y).astype(int)
    fx, fy = x - xi, y - yi
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
    a = lat[yi[:, None] % py, xi[None, :] % px]
    b = lat[yi[:, None] % py, (xi[None, :] + 1) % px]
    c = lat[(yi[:, None] + 1) % py, xi[None, :] % px]
    d = lat[(yi[:, None] + 1) % py, (xi[None, :] + 1) % px]
    top = a + (b - a) * fx[None, :]
    bot = c + (d - c) * fx[None, :]
    return top + (bot - top) * fy[:, None]


def fbm(n, px, py, octaves, seed, gain=0.5):
    s, a, tot = 0.0, 1.0, 0.0
    for o in range(octaves):
        s = s + a * pnoise(n, px * 2 ** o, py * 2 ** o, seed * 31 + o)
        tot += a
        a *= gain
    return s / tot


def blur(t, sigma):
    """A periodic Gaussian blur of a tile (sigma in texels)."""
    if sigma <= 0:
        return t
    n = t.shape[0]
    f = np.fft.fftfreq(n)
    g = np.exp(-2 * (math.pi * sigma * f) ** 2)
    return np.real(np.fft.ifft2(np.fft.fft2(t) * g[:, None] * g[None, :]))


def noise_table(name, px, py, octaves, seed, n=1000, levels=(0,), gain=0.5):
    """fbm 0..999 over a tile of n x n texels (rows y, columns x), and blurred copies for coarser footprints."""
    t = fbm(n, px, py, octaves, seed, gain)
    for k, s in enumerate(levels):
        put(f"{name}{k}" if len(levels) > 1 else name, np.clip(blur(t, s), 0, 0.999) * 1000)


# ---- the sky ----------------------------------------------------------------------------------------------------

DAY_SUN = np.array([-0.30, 0.86, -0.40]); DAY_SUN /= np.linalg.norm(DAY_SUN)
DUSK_SUN = np.array([-0.33, 0.10, 1.0]); DUSK_SUN /= np.linalg.norm(DUSK_SUN)
MAP_AZ = np.arange(-1400, 1401, 5)           # the sky's map: azimuths (thousandths of a radian from ahead) ...
MAP_EL = np.arange(-50, 701, 5)              # ... and the ray's height part (thousandths)
ROUND_AZ = np.arange(-3150, 3151, 50)        # the all-round map, for glass and metal
ROUND_EL = np.arange(0, 1001, 50)


def map_dirs(az, el):
    A, E = np.meshgrid(az / 1000, el / 1000)
    h = np.sqrt(np.clip(1 - E * E, 0, 1))
    return np.stack([h * np.sin(A), E, h * np.cos(A)], -1)


def sky_tables():
    for name, az, el in (("map", MAP_AZ, MAP_EL), ("round", ROUND_AZ, ROUND_EL)):
        d = map_dirs(az, el)
        put(name + "_dx", np.round(d[..., 0] * 999))
        put(name + "_dy", np.round(d[..., 1] * 999))
        put(name + "_dz", np.round(d[..., 2] * 999))
        put(name + "_az", np.tile(az, (len(el), 1)))
    i = np.arange(1000) / 1000
    put("sqrt", np.sqrt(i) * 1000)
    # the day: the sun's glow by the ray's dot with it (thousandths), times 250 (R), each channel
    near = np.clip(np.arange(-1000, 1001) / 1000, 0, 1)
    g = near ** 300 * 20 + near ** 12 * 0.35
    for c, k in enumerate((1.0, 0.92, 0.75)):
        put(f"glowday{c}", np.minimum(g * k * 250, 999))
    # the cloud plane: 1300 m (day) or 1800 m (dusk) over the height part, metres
    dy = np.maximum(np.arange(1000), 1)
    put("cloudk_day", np.minimum(1300 * 1000 // dy, 99_999))
    put("cloudk_dusk", np.minimum(1800 * 1000 // dy, 99_999))
    put("smooth_cover_day", smooth(0.52, 0.72, i) * 1000)
    put("smooth_low_day", smooth(0.015, 0.09, i) * 1000)
    put("smooth_far_day", smooth(0.0, 0.25, i) * 1000)
    # day clouds: cells of 1000 m over a 10 km tile at 10 m; the light side (toward the sun) as its own table
    n = fbm(1000, 10, 10, 5, 7)
    shift = np.roll(np.roll(n, 6, 0), 4, 1)             # the sun's way, 0.15 of a cell: 60 m in z, 45 in x
    lit = np.clip(0.75 - (shift - n) * 5.0, 0, 1)
    for k, s in enumerate((0, 2, 6)):
        put(f"cloud_day{k}", np.clip(blur(n, s), 0, 0.999) * 1000)
        put(f"cloudlit_day{k}", np.clip(blur(lit, s), 0, 0.999) * 1000)
    # the dusk: the gradient's mix by the height part (t = el / 0.45): low to mid below 0.3, mid to high above
    el = np.arange(-50, 1000)
    t = np.clip(el / 450, 0, 1)
    put("dusk_lowmid", np.round(np.clip(t / 0.3, 0, 1) * 100))
    put("dusk_midhigh", np.round(np.clip((t - 0.3) / 0.7, 0, 1) ** 0.8 * 100))
    put("dusk_islow", (t < 0.3).astype(int))
    # toward the sun's azimuth warm: ((cos(az - az_sun) + 1) / 2)^1.5, percent, by azimuth (thousandths of a radian, from -3150)
    azs = math.atan2(DUSK_SUN[0], DUSK_SUN[2])
    a = np.arange(-3150, 3151) / 1000
    put("dusk_warm", np.round(((np.cos(a - azs) + 1) / 2) ** 1.5 * 100))
    g = near ** 120 * 2.5 + near ** 8 * 0.45
    for c, k in enumerate((1.0, 0.62, 0.28)):
        put(f"glowdusk{c}", np.minimum(g * k * 250, 999))
    put("pow5", near ** 5 * 1000)
    eln = np.arange(1000) / 1000
    put("dusk_band", smooth(0.02, 0.10, eln) * (1 - 0.6 * smooth(0.45, 0.8, eln)) * 1000)
    put("smooth_dens", smooth(0.50, 0.78, i) * 1000)
    put("dusk_thin", (1 - smooth(0.5, 0.85, i)) * 1000)
    put("smooth_hl", smooth(0.0, 0.25, eln) * 1000)
    # dusk clouds: x cells 1429 m (7 a tile), z cells 1250 m (8), warped in x by 0.3 of a coarser noise (cells
    # 2500 x 5000 m)
    xs = (np.arange(1000) + 0.5) / 1000
    warp = fbm(1000, 4, 2, 2, 11)
    base = np.zeros((1000, 1000))
    for o in range(5):
        px, py = 7 * 2 ** o, 8 * 2 ** o
        # the warp moves x by 0.3 of a cell of the first octave
        sx = (xs[None, :] + 0.3 * warp / 7) % 1.0
        nn = pnoise(1000, px, py, 23 * 31 + o)
        cols = np.clip((sx * 1000).astype(int), 0, 999)
        base += 0.5 ** o * np.take_along_axis(nn, cols, 1)
    base /= sum(0.5 ** o for o in range(5))
    for k, s in enumerate((0, 2, 6)):
        put(f"cloud_dusk{k}", np.clip(blur(base, s), 0, 0.999) * 1000)
    # a footprint level for the clouds by the height part: more blur toward the horizon
    put("cloud_level", np.where(dy > 250, 0, np.where(dy > 90, 1, 2)))
    # directions to azimuths: atan of the ratio across / ahead (thousandths, -4000..4000), thousandths of a radian
    r = np.arange(-4000, 4001) / 1000
    put("atan", np.round(np.arctan(r) * 1000))
    put("recip_ahead", np.minimum(1_000_000 // np.maximum(np.arange(1000), 1), 999_999))
    # the far silhouettes by azimuth (thousandths of a radian, from -3150): heights as the ray's height part
    az = np.arange(-3150, 3151) / 1000
    FUJI_AZ, FUJI_H, FUJI_W = 0.36, 0.075, 0.30
    da = np.abs(az - FUJI_AZ) / FUJI_W
    fuji = np.minimum(FUJI_H * np.clip(1 - da, 0, 1) ** 1.7, FUJI_H * 0.93)
    def line_noise(freq, octaves, seed):
        """fbm along a line: value noise at freq cells a radian."""
        out = np.zeros_like(az)
        tot = 0
        for o in range(octaves):
            f = freq * 2 ** o
            lat = np.random.default_rng(seed + o).random(int(7 * f) + 3)
            x = (az + 3.5) * f
            xi = np.floor(x).astype(int); fx = x - xi; fx = fx * fx * (3 - 2 * fx)
            out += 0.5 ** o * (lat[xi] + (lat[xi + 1] - lat[xi]) * fx)
            tot += 0.5 ** o
        return out / tot
    hills = 0.010 + 0.016 * line_noise(6, 3, 41) * smooth(-0.2, 0.6, np.abs(az + 0.1))
    trees = 0.0035 + 0.004 * line_noise(90, 2, 43)
    put("ridge_fuji", np.round(np.sin(fuji) * 10000))
    put("ridge_hills", np.round(np.sin(hills) * 10000))
    put("ridge_trees", np.round(np.sin(trees) * 10000))
    put("ridge_snow", np.round(np.sin(fuji - FUJI_H * (0.30 + 0.05 * np.sin(az * 140))) * 10000))
    put("ridge_lit", np.round(np.clip(0.5 - (az - FUJI_AZ) * 6, 0, 1) * 100))
    # the sun's core at dusk, by the squared distance of the ray from the sun's way (millionths): 60 near^2000,
    # near = 1 - d2 / 2; R (x 250) split into a card and tenths beyond
    d2 = np.arange(20000) / 1e6
    core = 60 * np.clip(1 - d2 / 2, 0, 1) ** 2000
    for c, k in enumerate((1.0, 0.62, 0.28)):
        v = core * k * 250
        put(f"core{c}", np.minimum(v, 999))
        put(f"corehot{c}", np.maximum(v - 999, 0) / 25)


def post_tables():
    # the tone curve (the prototype's tonemap, before its saturation step): by R (250 for 1.0), thousandths
    x = np.arange(30000) / 250
    a = x * (2.51 * x + 0.03) / (x * (2.43 * x + 0.59) + 0.14)
    a = np.clip(a, 0, 1) ** (1 / 2.2)
    put("tone", np.minimum(a ** 1.18 * 1000, 999))
    # the tone curve with the vignette: by the vignette's level (factor (999 - 8 level - 4) / 1000) and R, 0..255
    lev = np.arange(32)[:, None]
    xv = (np.arange(40000)[None, :] / 250) * ((999 - 8 * lev - 4) / 1000)
    av = xv * (2.51 * xv + 0.03) / (xv * (2.43 * xv + 0.59) + 0.14)
    put("tonev", np.round(np.clip(av, 0, 1) ** (1 / 2.2 * 1.18) * 255))
    # exposure's second ramp to the 0.7th power (thousandths)
    put("pow07", (np.arange(1001) / 1000) ** 0.7 * 1000)


# ---- the materials ----------------------------------------------------------------------------------------------

LEVELS = 80                                   # footprint levels: quarter octaves of millimetres


def fw_cm(L):
    return 2 ** (L / 4) / 10


def fstripe(u, period, duty, fw):
    fw = np.maximum(fw, 1e-4)
    a, b = (u - fw / 2) / period, (u + fw / 2) / period
    F = lambda x: np.floor(x) * duty + np.minimum(x - np.floor(x), duty)
    return np.clip((F(b) - F(a)) / (b - a), 0, 1)


def fband(u, c, hw, fw):
    fw = np.maximum(fw, 1e-4)
    lo, hi = np.maximum(u - fw / 2, c - hw), np.minimum(u + fw / 2, c + hw)
    return np.clip((hi - lo) / fw, 0, 1)


def stripe(name, period, duty, shift=0.0):
    """The share of a footprint on a stripe, by the footprint's level and the place in the period (thousandths),
    percent; the stripe from `shift` (centimetres) into the period."""
    u = (np.arange(1000) + 0.5) / 1000 * period - shift
    L = np.arange(LEVELS)[:, None]
    put(name, np.round(fstripe(u[None, :], period, duty, fw_cm(L)) * 100))


def band(name, hw):
    """The share of a footprint on a band hw either side of its middle, by level and distance (centimetres)."""
    d = np.arange(1000)
    L = np.arange(LEVELS)[:, None]
    put(name, np.round(fband(d[None, :], 0, hw, fw_cm(L)) * 100))


def noise10(name, cx, cy, octaves, seed, gain=0.5):
    """fbm over a 10 m tile at a centimetre (cx by cy cells), rows along the second axis."""
    put(name, np.clip(fbm(1000, cx, cy, octaves, seed, gain), 0, 0.999) * 1000)


def tint(name, rgb, a, b, src=None):
    """An albedo times (a + b n) by a noise value n 0..999, thousandths, each channel."""
    n = np.arange(1000) / 1000
    for c in range(3):
        put(f"{name}{c}", np.clip(rgb[c] * (a + b * n), 0, 0.999) * 1000)


def material_tables():
    L = np.arange(100_000)
    put("lev", np.clip(np.round(4 * np.log2(np.maximum(L, 1))), 0, LEVELS - 1))
    # 1 / max(cos, 0.08), tenths, by the ray's dot with the normal (thousandths)
    c = np.arange(1000) / 1000
    put("invcos", np.round(10 / np.maximum(c, 0.08)))
    # by the depth's logarithm (l = ln K / z, ten-thousandths; K / z = 42,814,286 / (z / 0.7)): the pixel's width
    # there (mm, z / 0.7 times 7 / 950) and the levels of the footprints across (the width), along the deck (the width
    # times z over the eye's 3 m) and the paddies (over 6 m), three widths; where the width passes 60 mm
    l = np.arange(138_200)
    z9 = 42_814_286 / np.exp(l / 10000)
    fpx = z9 * 7 / 950
    zt = z9 * 7 / 10
    lev = lambda mm: np.clip(np.round(4 * np.log2(np.maximum(np.clip(np.floor(mm), 0, 99_999), 1))), 0, LEVELS - 1)
    put("lev_px", lev(fpx))
    put("lev_deck", lev(np.minimum(fpx, 999) * np.minimum(zt, 50_000) / 300))
    put("lev_paddy", lev(np.minimum(fpx, 999) * np.minimum(zt, 50_000) / 600))
    put("lev_px3", lev(3 * fpx))
    put("far_px60", fpx > 60)
    # the footprint's level by ln(width / cosine): ln (invcos / 10) + ln ((KZ / 0.7) 7 / 950) (ten-thousandths) by the dot,
    # less l; the level by that
    put("ln_foot_ic", np.floor(np.log(np.round(10 / np.maximum(c, 0.08)) / 10) * 10000) + round(math.log(42_814_286 * 7 / 950) * 10000))
    put("lev_ln", lev(np.exp(np.arange(116_000) / 10000)))
    # logarithms (ten-thousandths) of depths K / z and of rays' sizes plus ln ((KZ / 0.7) / 1000); e back (centimetres, at
    # most 870,000, so a place stays a long; 0 for 0) and its negative; a ray's sign
    n = np.arange(1_000_000)
    put("lniz4", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 10000), 0))
    r = np.abs(np.arange(-999, 1000))
    put("lnr4", np.where(r > 0, np.floor(np.log(np.maximum(r, 1)) * 10000) + round(math.log(42_814.286) * 10000), -800_000))
    k = np.arange(140_000)
    ex = np.where(k > 0, np.minimum(np.floor(np.exp(k / 10000)), 870_000), 0)
    put("exp4", ex); put("exp4n", -ex)
    put("exp4c", np.minimum(ex, 999)); put("exp4cn", -np.minimum(ex, 999))
    r = np.abs(np.arange(-999, 1000))
    put("lnr0", np.where(r > 0, np.floor(np.log(np.maximum(r, 1)) * 10000), -500_000))
    # the same for slopes (thousandths a place), up to three cards
    k = np.arange(208_000)
    ex = np.where(k > 0, np.minimum(np.floor(np.exp(k / 10000)), 998_999_999), 0)
    put("exps4", ex); put("exps4n", -ex)
    put("neg", np.arange(-999, 1000) < 0)
    # for slopes: ln |d| + ln 1000 for changes d from -999,999 (ten-thousandths; -500,000 for 0) and their signs
    d = np.abs(np.arange(-999_999, 1_000_000))
    put("lnd4", np.where(d > 0, np.floor(np.log(np.maximum(d, 1)) * 10000) + round(math.log(1000) * 10000), -500_000))
    put("negd", np.arange(-999_999, 1_000_000) < 0)
    # for edges' crossings: ln (99 - r) less ln 1000, by r 0..99 (ten-thousandths)
    r = np.arange(100)
    put("ln_off4", np.where(r < 99, np.floor(np.log(np.maximum(99 - r, 1)) * 10000) - round(math.log(1000) * 10000), -250_000))
    put("hash", np.random.default_rng(1964).integers(0, 1000, (1000, 1000)))
    # the deck: across the line (centimetres from -1000), the slabs, sleepers' band, the rails' darkening and the
    # rails, by the footprint's level
    xr = np.arange(-1000, 1000)[None, :]
    Lc = fw_cm(np.arange(LEVELS))[:, None]
    slab = sum(fband(xr, c0, 117, Lc) for c0 in (0, 430))
    slp = sum(fband(xr, c0, 125, Lc) for c0 in (0, 430))
    dark = np.ones_like(slab); rail = np.zeros_like(slab)
    for c0 in (0, 430):
        for rx in (c0 - 71.75, c0 + 71.75):
            dark = dark * (1 - 0.75 * fband(xr, rx, 11, Lc))
            rail = np.maximum(rail, fband(xr, rx, 3.4, Lc))
    put("deck_slab", np.round(slab * 100)); put("deck_slp", np.round(slp * 100))
    put("deck_dark", np.round(dark * 100)); put("deck_rail", np.round(rail * 100))
    stripe("deck_joint", 1e5 / 202, 0.03)
    stripe("deck_sleeper", 1e4 / 167, 0.38, -10)
    noise10("n_deck", 60, 60, 2, 101)
    noise10("n_gravel", 180, 180, 2, 102)
    noise10("n_verge", 15, 15, 3, 103)
    tint("deck_conc", (0.30, 0.295, 0.285), 0.85, 0.3)
    tint("deck_gravel", (0.30, 0.27, 0.24), 0.6, 0.8)
    tint("deck_slabc", (0.22, 0.22, 0.215), 0.9, 0.2)
    tint("deck_verge", (0.11, 0.15, 0.06), 0.6, 0.8)
    # the paddies: dikes at the cells' edges (31.25 by 18.87 m cells, by the place in the cell), the planting rows
    # (30 cm) and their dashes (18 cm), the farm road's band
    for name, per in (("dike_u", 3125), ("dike_v", 1887)):
        u = (np.arange(1000) + 0.5) / 1000 * per
        Lr = np.arange(LEVELS)[:, None]
        put(name, np.round(np.clip(fband(u[None, :], 55, 55, fw_cm(Lr)) + fband(u[None, :], per - 55, 55, fw_cm(Lr)), 0, 1) * 100))
    stripe("paddy_row", 1e4 / 333, 0.22)
    stripe("paddy_dash", 1e4 / 556, 0.55)
    put("paddy_wobble", np.round(70 * np.sin((np.arange(1000) / 1000) * 18.87 * 0.4) / 30.03))
    band("band_road", 220)
    noise10("n_grn", 8, 8, 2, 104)
    noise10("n_soil", 20, 20, 2, 105)
    noise10("n_grass", 15, 15, 2, 106)
    tint("paddy_grn", (0.12, 0.26, 0.05), 0.7, 0.6)
    tint("paddy_soil", (0.20, 0.14, 0.09), 0.8, 0.4)
    tint("paddy_grass", (0.13, 0.20, 0.05), 0.75, 0.5)
    tint("paddy_seed", (0.10, 0.22, 0.04), 0.8, 0.4)
    # the paddies' ripples: 100 m tiles at 10 cm, cells 2.86 by 8.33 m and 8.33 by 2.86 m; then a normal's lean
    # (thousandths, 0.035 for 1)
    put("n_ripx", (fbm(1000, 35, 12, 2, 107) - 0.5) * 35)
    put("n_ripz", (fbm(1000, 12, 35, 2, 108) - 0.5) * 35)
    # Schlick's reflectance, f0 0.02, by the ray's height part (thousandths, -1000..0): percent of a thousand
    d = np.arange(1001) / 1000
    put("schlick02", np.round((0.02 + 0.98 * (1 - d) ** 5) * 1000))
    for f0, nm in ((0.25, 25), (0.06, 6), (0.12, 12), (0.08, 8), (0.1, 10), (0.045, 45)):
        put(f"schlick{nm}", np.round((f0 + (1 - f0) * (1 - d) ** 5) * 1000))
    noise10("n_embank", 12, 12, 3, 109)
    noise10("n_bank", 9, 25, 3, 110)
    tint("embank", (0.14, 0.21, 0.06), 0.7, 0.6)
    tint("bank", (0.16, 0.18, 0.08), 0.6, 0.7)
    # the river: ripples over 100 m tiles (thousandths of a normal's lean)
    put("n_rivx", (fbm(1000, 8, 50, 3, 111) - 0.5) * 140 + (fbm(1000, 60, 250, 1, 112) - 0.5) * 50)
    put("n_rivz", (fbm(1000, 50, 8, 3, 113) - 0.5) * 100)
    # the sun's glint on the water by the reflected ray's squared distance from the sun's way (millionths): 40 near^900
    # + 1.5 near^90, times 3 (the Fresnel at its least) times the sun's colour; R by the card and tenths beyond
    d2 = np.arange(100_000) / 1e6
    near = np.clip(1 - d2 / 2, 0, 1)
    gl = near ** 900 * 40 + near ** 90 * 1.5
    for c, k in enumerate((2.6, 1.05, 0.35)):
        put(f"glint{c}", np.minimum(gl * k * 250, 999_999))
    # buildings: floors 3.6 m (window rows, the curtain wall's spandrels, balcony fronts), window columns by pitch,
    # mullions 1.6 m; the per-window lintel shadow and blinds
    stripe("bld_winv", 1e5 / 278, 0.55)
    stripe("bld_floor88", 1e5 / 278, 0.88)
    stripe("bld_balc", 1e5 / 278, 0.32)
    stripe("bld_winu", 290, 0.62)
    stripe("bld_wina", 406, 0.8)
    stripe("bld_mull", 160, 0.92)
    v = np.arange(1000) / 1000 / 0.55
    put("bld_lintel", np.round(np.clip((v - 0.8) / 0.12, 0, 1) * 100))
    put("n_bldvar", fbm(1000, 5, 10, 1, 114) * 999)
    # weathering: streaks (cells 0.77 m across, 20 m up; 100 m tiles at 10 cm up, 1 cm across), drips under the top,
    # dirt over the foot, blotches (4 m cells, 100 m tile)
    put("n_streak", np.clip(smooth(0.5, 0.85, fbm(1000, 13, 5, 3, 115)) * 1000, 0, 999))
    put("n_drip", fbm(1000, 21, 30, 1, 116) * 999)
    put("n_blot", np.clip(smooth(0.55, 0.9, fbm(1000, 25, 25, 2, 117)) * 1000, 0, 999))
    h = np.arange(1000) * 10                       # centimetres
    put("w_drip", np.exp(-h / 220) * 1000)
    put("w_dirt", np.exp(-h / 120) * 1000)
    put("w_streak_top", (0.5 + 0.5 * smooth(0, 600, -h + 300)) * 1000)
    put("canyon", np.round((0.45 + 0.55 * smooth(0, 4650, np.arange(1000) * 10)) * 100))
    noise10("n_rooftop", 5, 5, 1, 118)
    noise10("n_conc", 4, 9, 3, 119)
    stripe("conc_joint", 1000, 0.02)
    put("conc_jl", np.round(1e5 / np.maximum(np.arange(1000), 50)))
    put("n_hill", fbm(1000, 28, 28, 3, 120) * 999)
    put("n_hillvar", fbm(1000, 5, 5, 1, 121) * 999)
    put("hill_gaps", np.round(smooth(0.25, 0.55, np.arange(1000) / 1000) * 1000))
    stripe("tun_ring", 1050, 0.015)
    noise10("n_tunnel", 15, 6, 3, 122)
    noise10("n_metal", 10, 20, 1, 123)
    noise10("n_wood", 30, 30, 1, 124)
    noise10("n_shed", 4, 4, 3, 125)
    put("shed_rust", np.round(smooth(0.5, 0.8, np.arange(1000) / 1000) * 1000))
    stripe("shed_cor", 20, 0.5)
    noise10("n_tree1", 9, 9, 3, 126)
    noise10("n_tree2", 16, 16, 3, 127)
    noise10("n_tree3", 22, 22, 3, 128)
    put("n_treevar", fbm(1000, 5, 5, 1, 129) * 999)
    noise10("n_cut1", 17, 17, 1, 130)
    noise10("n_cut2", 46, 46, 1, 131)
    put("cut_need", np.round((0.12 + 0.75 * (1 - np.arange(1000) / 1000) ** 2) * 1000))
    put("pow6", near_tab(6)); put("pow3", near_tab(3))
    put("sq", (np.arange(1001) / 1000) ** 2 * 1000); put("cube", (np.arange(1001) / 1000) ** 3 * 1000)
    # suburb houses: windows by pitch 3.2 m, storeys 2.85 m, siding
    stripe("ch_win", 320, 0.45, -64)
    v = (np.arange(1000) + 0.5) / 1000 * 285
    Lr = np.arange(LEVELS)[:, None]
    put("ch_winv", np.round(fband(v[None, :], 155, 55, fw_cm(Lr)) * 100))
    put("ch_rail", np.round(fband(v[None, :], 95, 6, fw_cm(Lr)) * 100))
    stripe("ch_siding_h", 21, 0.12)
    stripe("ch_siding_v", 45, 0.08)
    stripe("solar_x", 100, 0.04)
    stripe("solar_z", 165, 0.03)
    stripe("house_post", 182, 0.08)
    stripe("house_win", 364, 0.5, -40)
    band("band_housewin", 60)
    put("house_lower", np.round(smooth(110, 95, np.arange(1000)) * 100))
    stripe("tile_rows", 1e4 / 300, 0.25)
    # the train: its window rows (1.05 m), the nose's stripe and its height down the nose, the canopy's crease
    stripe("train_win", 105, 0.72, -30)
    band("band_plat16", 16); band("band_plat8", 8)
    # the departure board: its dots (128 x 40, the IBM VGA 8x16 font: 1 amber, 2 green; a border of none round
    # them is the clamp's) and a dot's shape across a 7 cm cell by the footprint's level (percent)
    from PIL import Image, ImageDraw, ImageFont
    f = ImageFont.truetype(os.path.join(HERE, "..", "..", "..", "prototypes", "pcfonts", "Px437_IBM_VGA_8x16.ttf"), 16)
    im = Image.new("L", (128, 40), 0)
    d = ImageDraw.Draw(im)
    for i_, s_ in enumerate(["14:03 NOZOMI 37", "FOR SHIN-OSAKA"]):
        d.text((0, 3 + i_ * 18), s_, font=f, fill=255)
    bits = (np.array(im) > 127).astype(np.int64)
    bits[20:] *= 2
    put("led_bits", np.pad(bits, 1))
    fr = np.arange(29) / 29
    L = np.arange(LEVELS)[:, None]
    put("led_dot", np.round(fband(fr[None, :], 0.5, 0.36, fw_cm(L) / 7.25) * 100))
    put("led_fill", np.round(6.25 * np.clip(fw_cm(np.arange(LEVELS)) / 7.25 - 0.5, 0, 1)))
    band("band_ns1", 16); band("band_ns2", 3.6)
    dn = np.arange(3000)
    put("train_sy", np.round((1.15 - 0.62 * (1 - smooth(0.5, 10.7, dn / 100))) * 100))
    s_ = np.clip(dn / 1070, 0, 1)
    put("train_hwn", np.round(169 * (0.60 + 0.40 * (1 - (1 - s_) ** 2.4)) * 0.46))
    r = np.arange(3000) / 1000
    put("train_crease", np.round(np.exp(-((r - 0.95) / 0.18) ** 2) * 100))
    put("train_rimy", np.round(smooth(0.2, 0.7, np.arange(-127, 128) / 100) * 100))
    # fog: e^(-d / 1400 m) by day, e^(-d / 3200 m) at dusk, by metres, percent
    d = np.arange(20000)
    put("fog_day", np.round(np.exp(-d / 1400) * 100)); put("fog_dusk", np.round(np.exp(-d / 3200) * 100))
    # the same by the distance's logarithm (thousandths) plus 3000, and logarithms of depths K / z (thousandths)
    d = np.clip(np.floor(np.exp((np.arange(13000) - 3000) / 1000)), 0, 19999)
    put("fogl_day", np.round(np.exp(-d / 1400) * 100)); put("fogl_dusk", np.round(np.exp(-d / 3200) * 100))
    n = np.arange(1_000_000)
    put("lniz", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 1000), -20000))
    # the ground's stretches by the place along the line (centimetres from -100,000), and for each stretch the deck's
    # half width, whether it is the bridge's, the river's (with its banks), the water's, the river's near half
    TUN0, TUN1, BR0, BR1, RIV0, RIV1, BANK = 26_000, 50_000, 92_000, 104_000, 92_800, 103_200, 1000
    edges = [TUN0 - 6000, TUN0, TUN1, RIV0 - BANK, RIV0 - BANK + 1, BR0, BR0 + 1, RIV0, RIV0 + 1, (RIV0 + RIV1) // 2, RIV1, BR1, RIV1 + BANK, RIV1 + BANK + 1]
    z = np.arange(-100_000, 1_000_000)
    zone = np.searchsorted(np.array(edges), z, side="right")
    put("g_zone", zone)
    mids = [edges[0] - 1] + [e for e in edges]
    hw = [570 if m < TUN0 - 6000 else 1200 if m < TUN0 else 500 if m < TUN1 else 590 if BR0 <= m < BR1 else 500 for m in mids]
    put("g_hw", np.array(hw))
    put("g_bridge", np.array([int(BR0 < m < BR1) for m in mids]))
    put("g_river", np.array([int(RIV0 - BANK < m < RIV1 + BANK) for m in mids]))
    put("g_water", np.array([int(RIV0 < m < RIV1) for m in mids]))
    put("g_mid", np.array([int(m < (RIV0 + RIV1) // 2) for m in mids]))
    # 25 times a card (the tone's index), and differences -510..510 over 20 cut toward zero (the tone's push)
    put("x25", np.arange(1000) * 25)
    d = np.arange(-510, 511)
    put("div20", np.trunc(d / 20))
    # the screen's numbers by a pixel's squared distance from the middle s (pixels squared; the half-pixel offsets'
    # squares are 4 s + 2): q = 1,900,000 over the ray's length (half pixels, at most 999), the vignette (thousandths)
    # and its level for the tone ((999 - it) / 8), q / 1.4 (q7), and for the fog ln (700,000 / q) + ln 428.14 + 3
    # (thousandths)
    s = np.arange(737_281)
    s2 = 4 * s + 2
    q = np.minimum(np.floor(1_900_000 / np.floor(np.sqrt(s2 + 1900 * 1900))), 999)
    vig = 999 - np.floor(np.floor(s2 * 22 / 1000) / 295)
    put("scr_vlev", np.floor((999 - vig) / 8))
    put("scr_nlq7", np.floor(np.log(500 / np.floor(q * 1000 / 1400)) * 10000))
    qf = np.floor(700_000 / q)
    put("scr_lq", np.floor(np.log(qf) * 1000) + 6059 + 3000)
    # the projection's: ln (z / 10) + ln (1000 / 9500) for a full screen, + ln (1000 / 4750) for a half-size one
    # (ten-thousandths; so ln |x| + ln 1000 less it is ln of x 9500 / (z / 10), hundredths of a pixel), and K over
    # z / 10 (and K / 10 for places in centimetres)
    n = np.arange(1_000_000)
    lnn = np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 10000), 0)
    put("lnz_f", lnn + round(math.log(1000 / 9500) * 10000))
    put("lnz_f2", lnn + round(math.log(1000 / 4750) * 10000))
    put("lnz_f4", lnn + round(math.log(1000 / 2375) * 10000))
    put("kz_z10", np.where(n > 0, np.minimum(29_970_000 // np.maximum(n, 1), 999_999), 999_999))
    put("kz10_z10", np.where(n > 0, np.minimum(2_997_000 // np.maximum(n, 1), 999_999), 999_999))
    # for making normals units: squares of -577..577, half the ln of a sum of squares, and ln |c| + ln 99 (ten-
    # thousandths; -500,000 for 0)
    c = np.arange(-577, 578)
    put("sq577", c * c)
    n = np.arange(1_000_000)
    put("half_ln4", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 5000), 0))
    put("ln99c4", np.where(c != 0, np.floor(np.log(np.maximum(np.abs(c), 1)) * 10000) + round(math.log(99) * 10000), -500_000))
    # squares of -999..999, held to 20,000 (the sun's core)
    c = np.arange(-999, 1000)
    put("sq999c", np.minimum(c * c, 20_000))
    # ten times 0..99
    put("x10", np.arange(100) * 10)
    # how many 25ths
    put("div25", np.arange(1000) // 25)
    put("pow4", near_tab(4))
    put("leak", np.exp(-np.arange(1000) * 10 / 1200) * 1000)


def tunnel_tables():
    """The places the tunnel's light maps are worked out at (centimetres, the line's middle MID = 215 across): the
    lamps' map every 10 cm along a lamp's 12 m and 20 cm across and up; the headlights' every 50 cm ahead over 100 m;
    each place's normal (hundredths): up on the deck and the troughs, else the lining's, toward the section's middle.
    Only the places near the tunnel's surfaces, and each one's index in the map."""
    MID, YC = 215, 350
    for name, nz, dz in (("lampgrid", 120, 10), ("headgrid", 200, 50)):
        zi, yi, xi = np.meshgrid(np.arange(nz), np.arange(48), np.arange(63), indexing="ij")
        x = MID - 620 + 20 * xi + 10
        y = 20 * yi - 50 + 10
        z = dz * zi + dz // 2
        deck = (y < 60) & (np.abs(x - MID) < 490)
        ny = np.where(deck, 1.0, np.where(y > YC, YC - y, 0.0))
        nx = np.where(deck, 0.0, MID - x)
        ln = np.maximum(np.hypot(nx, ny), 1e-9)
        # the places near the tunnel's surfaces (the deck, the walls and troughs, the lining and the portal's face beyond
        # it, the brackets in the roof; generously, as the places seen come a little off them): only those worked out
        near = (y < 100) | ((np.abs(x - MID) > 380) & (y <= YC + 80)) | ((y > YC - 80) & (np.hypot(x - MID, y - YC) > 380)) \
            | (((np.abs(x) < 130) | (np.abs(x - 430) < 130)) & (y > 520) & (y < 950))
        keep = near.reshape(-1)
        for k, a in (("x", x), ("y", y), ("z", z), ("nx", np.round(nx / ln * 99)), ("ny", np.round(ny / ln * 99))):
            put(f"{name}_{k}", a.reshape(-1)[keep])
        put(f"{name}_at", np.nonzero(keep)[0])
    # the lamps' light by the squared distance (tenths of a square decimetre): 1.3 / (|v| (d2 + 3)), m, times 250,
    # per decimetre of v.n, thousandths
    d2 = (np.arange(30000) * 10 + 5) / 100
    put("lamp_g", np.minimum(1.3 * 250 / (np.sqrt(d2) * (d2 + 3)) / 10 * 1000, 999))
    # the headlights: 60 cone / (|v| (d2 + 1)) per decimetre of v.n, the cone by -l.dir (thousandths)^30
    put("head_g", np.minimum(60 * 250 / (np.sqrt(d2) * (d2 + 1)) / 10 * 1000, 999_999))
    c = np.clip(np.arange(1001) / 1000, 0, 1)
    put("head_cone", c ** 30 * 1000)


def blur_tables():
    """Lines across the screen in 16 directions (over a half turn, from across), for running sums along them:
    for each direction and pixel, its line and its place along it (lines step one pixel along their longer axis each
    place); each line's length. The steps' length (thousandths of a pixel per place) by direction."""
    W, H = 1536, 768
    ys, xs = np.mgrid[0:H, 0:W]
    lines, places, lens, step = [], [], [], []
    for d in range(16):
        a = math.pi * d / 16
        c, sn = math.cos(a), math.sin(a)
        if abs(c) >= abs(sn):
            t = sn / c                                   # along x: y = y0 + round(x t)
            off = np.round(xs * t).astype(int)
            y0 = ys - off
            lid = y0 - y0.min()
            pos = xs
            step.append(round(1000 / abs(c)))
        else:
            t = c / sn                                   # along y: x = x0 + round(y t)
            off = np.round(ys * t).astype(int)
            x0 = xs - off
            lid = x0 - x0.min()
            pos = ys
            step.append(round(1000 / abs(sn)))
        # each line's places from its first pixel
        first = np.full(lid.max() + 1, 10 ** 9)
        np.minimum.at(first, lid.ravel(), pos.ravel())
        last = np.full(lid.max() + 1, -1)
        np.maximum.at(last, lid.ravel(), pos.ravel())
        lines.append(lid.ravel()); places.append((pos - first[lid]).ravel()); lens.append(last - first + 1)
    put("blur_line", np.stack(lines))
    put("blur_place", np.stack(places))
    m = max(len(l) for l in lens)
    put("blur_len", np.stack([np.pad(l, (0, m - len(l))) for l in lens]))
    put("blur_step", np.array(step))
    # the direction (0..15) by the velocity's angle: atan2 by the ratio of the smaller part over the larger
    # (thousandths) and which is larger; a half turn, so signs only as their product
    r = np.arange(1001) / 1000
    ang = np.arctan(r)                                  # 0..45 degrees
    put("blur_dir_x", np.round(ang / (math.pi / 16)) % 16)          # x the larger, same signs
    put("blur_dir_y", np.round((math.pi / 2 - ang) / (math.pi / 16)) % 16)
    # means by logarithms: ln of a sum (ten-thousandths; of 0, -1) and e back (the floor), so a mean is
    # exp[ln[sum] - ln[n]] with no division
    n = np.arange(400_000)
    put("ln", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 10000), -1))
    e = np.floor(np.exp(np.arange(70_000) / 10000) + 1e-9)
    put("exp", np.concatenate([[0], np.minimum(e, 999)]))
    e = np.floor(np.exp(np.arange(80_000) / 10000) + 1e-9)
    put("exp_long", np.concatenate([[0], e]))
    # ln of a count less one (so exp_long[ln[sum] - ln_m1[n]] is the mean), and the glow's brightness from a block's
    # four samples' sum (the card and 25 times the tenths each): what passes 1.3 R, in twentieths, at most 999
    n = np.arange(64)
    put("ln_m1", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 10000) - 1, 0))
    s = np.arange(4 * 25_974 + 1)
    put("bloom_br", np.minimum(np.maximum(s - 1300, 0) // 80, 999))
    # the glow's two blurs' sum (0..1998) as a card by its square root, and back four times (R)
    g = np.arange(1999)
    put("glow_enc", np.round(999 * np.sqrt(g / 1998)))
    e = np.arange(1000)
    put("glow_dec", np.round(4 * 1998 * (e / 999) ** 2))
    # a lens ghost's disc by the squared distance from its middle over its radius squared (thousandths):
    # (1 - smooth(0.75, 1, d)) (0.6 + 0.4 smooth(0, 1, d)), percent
    t = np.sqrt(np.arange(1500) / 1000)
    put("disc", np.round((1 - smooth(0.75, 1, t)) * (0.6 + 0.4 * smooth(0, 1, t)) * 100))
    # interleaved gradient noise, the prototype's jitter (thousandths), by pixel
    put("ign", ((52.9829189 * ((0.06711056 * xs + 0.00583715 * ys) % 1.0)) % 1.0) * 1000)


def wire_tables():
    """A wire's cover of a pixel by its width (rows, hundredths of a pixel) and the distance across (columns,
    hundredths): clip(w + 0.5 - d, 0, 1) min(1, 2 w), percent."""
    wv = np.arange(1000)[:, None] / 100
    d = np.arange(1000)[None, :] / 100
    put("wire_cov", np.round(np.clip(wv + 0.5 - d, 0, 1) * np.minimum(1, 2 * wv) * 100))


def overlay_tables():
    """The words over the city, as the prototype's overlays() draws them: the ray-generation shader (Menlo 19) and
    the brag (Helvetica Neue Bold 44), each as its ink (0..255 a pixel, at 1536x768) and the dark panel's box behind
    it (x0, y0, x1, y1)."""
    import sys
    from PIL import Image, ImageDraw, ImageFont
    sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "prototypes"))
    import train as T
    W, H, s = 1536, 768, 1.0
    lay = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(lay)
    f = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", int(19 * s))
    for i, line in enumerate(T.CODE):
        d.text((int(70 * s), int(70 * s) + int(27 * s) * i), line, font=f, fill=255)
    lay2 = Image.new("L", (W, H), 0)
    d2 = ImageDraw.Draw(lay2)
    fb = ImageFont.truetype("/System/Library/Fonts/HelveticaNeue.ttc", int(44 * s), index=1)
    y = H - int(170 * s)
    for i, line in enumerate(T.BRAG):
        d2.text((int(70 * s), y + int(54 * s) * i), line, font=fb, fill=255)
    boxes = []
    for name, im in (("ink_code", lay), ("ink_brag", lay2)):
        a = np.array(im, np.int64)
        put(name, a.reshape(-1))
        ys_, xs_ = np.nonzero(a > 0.3 * 255)
        boxes.append([max(0, xs_.min() - 26), max(0, ys_.min() - 22), min(W, xs_.max() + 26), min(H, ys_.max() + 22)])
    put("ink_box", np.array(boxes))


def sign_tables():
    """The signs' characters (the prototype's strokes): 64 characters of three level and three upright strokes, each
    on by chance with chance extents, on a 32 x 32 grid of the character's cell (down the sign, across), filtered
    for a footprint of 0.02 and of 0.15 of a cell; percent. Rows: character, columns: place (32 down times 32)."""
    rng = np.random.default_rng(1868)
    f = (np.arange(32) + 0.5) / 32
    FA, FB = np.meshgrid(f, f, indexing="ij")
    for lvl, fw in enumerate((0.02, 0.15)):
        tab = []
        r2 = np.random.default_rng(1868)
        for g in range(64):
            h = r2.random(12)
            glyph = np.zeros_like(FA)
            for k, yb in enumerate((0.18, 0.5, 0.82)):
                if h[k] > 0.4:
                    x0, x1 = 0.12 + 0.3 * h[3 + k], 0.58 + 0.3 * h[6 + k]
                    glyph = np.maximum(glyph, fband(FA, yb, 0.06, fw) * fband(FB, (x0 + x1) / 2, (x1 - x0) / 2, fw))
            h2 = r2.random(9)
            for k, xb in enumerate((0.28, 0.5, 0.72)):
                if h2[k] > 0.45:
                    y0, y1 = 0.1 + 0.35 * h2[3 + k], 0.55 + 0.35 * h2[6 + k]
                    glyph = np.maximum(glyph, fband(FB, xb, 0.06, fw) * fband(FA, (y0 + y1) / 2, (y1 - y0) / 2, fw))
            tab.append(np.round(glyph * 100).reshape(-1))
        put(f"glyph{lvl}", np.array(tab))
    _ = rng


def motion_tables():
    """The blur's: logarithms of a flow's parts and depths (thousandths), e back (floor, at most 400) for a speed
    from them; and by a block's speed across and down (held to -400..400; rows down, columns across): the speed held
    to 138 long, the line's direction (16 over a half turn), its length, and the places either way along the line."""
    n = np.arange(100_000)
    put("lnv", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 1000), -20000))
    i = np.arange(9000)
    put("expv", np.minimum(np.floor(np.exp((i - 2000) / 1000) + 1e-9), 400))
    put("expv_neg", -np.minimum(np.floor(np.exp((i - 2000) / 1000) + 1e-9), 400))
    put("lnk", np.array([round(math.log(299_700) * 1000) - 2000]))
    vv = np.arange(-400, 401)
    VY, VX = np.meshgrid(vv, vv, indexing="ij")
    m = np.sqrt(VX * VX + VY * VY)
    k = np.where(m > 138, 138 / np.maximum(m, 1e-9), 1.0)
    cx, cy = np.trunc(VX * k).astype(int), np.trunc(VY * k).astype(int)
    ln = np.floor(np.sqrt(cx * cx + cy * cy)).astype(int)
    a = np.arctan2(cy, cx)
    d = np.round(a / (math.pi / 16)).astype(int) % 16
    put("bl_len", ln)
    put("bl_dir", d)
    step = []
    for dd in range(16):
        aa = math.pi * dd / 16
        step.append(round(1000 / max(abs(math.cos(aa)), abs(math.sin(aa)))))
    put("bl_h", np.array([[l * 500 // step[dd] for l in range(200)] for dd in range(16)]))
    # means by logarithms: ln of a sum (ten-thousandths) plus 1000 (of 0: 0); e back by the difference with ln n:
    # 0 under 1000, else floor(e^((d - 1000) / 10000))
    n = np.arange(400_000)
    put("ln_s", np.where(n > 0, np.floor(np.log(np.maximum(n, 1)) * 10000) + 1000, 0))
    put("ln_n", np.floor(np.log(np.maximum(np.arange(2000), 1)) * 10000))
    i = np.arange(80_000)
    put("exp_s", np.where(i < 1000, 0, np.floor(np.exp((i - 1000) / 10000) + 1e-9)))


def near_tab(k):
    near = np.clip(np.arange(-1000, 1001) / 1000, 0, 1)
    return near ** k * 1000


def blur_half():
    """The motion blur's lines at half the resolution (768x384, the party version's world): as blur_tables'."""
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


def main():
    if "--half" in sys.argv:                      # data/tables_h.bin: only the half resolution's blur lines
        global OUT
        OUT = os.path.join(HERE, "tables_h.bin")
        blur_half()
        write()
        return
    sky_tables()
    post_tables()
    material_tables()
    tunnel_tables()
    blur_tables()
    wire_tables()
    overlay_tables()
    sign_tables()
    motion_tables()
    write()


def write():
    with open(OUT, "wb") as f:
        for name, a, w in ARR:
            f.write(records.record(name, a, (len(a) // max(w, 1) if w else len(a), max(w, 1) if w else 1)))
    print(len(ARR), "tables,", sum(a.size for _, a, _ in ARR), "numbers")


if __name__ == "__main__":
    main()
