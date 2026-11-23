"""The demo callouts: nods to the demoscene's demos on the screens, signs, banners, flags and
clothes of the train's night flight and the stadium. Shared: train_night.py uses it; the stadium can too.

The list is callouts_final.json in the source files' folder ($CARD_ATTACK_WORK, not in this repository) (63 entries, in the order the screens pick them by):
the 8 demos whose footage we may show (kind clip: demos/<slug>/frames_160x120.rgb, 60 frames at 12 fps; logo.png)
and 55 parodies (kind parody: a parody title and group over a stand-in effect worked out on CARDIAC, standin()).

Every look is drawn here from fonts and numbers (no copied logos): a period style per entry:
  pc90    the IBM VGA 8x16 font, ANSI colours (PC demos to 1999)
  pc00    Helvetica Neue, lower case, white on black, thin rules (PC 2000 on: Farbrausch, ASD, Conspiracy...)
  amiga   bold type in copper-bar gradients (Amiga)
  c64     the IBM CGA 8x8 font in the C64's light blue on blue (C64)
  cga     the CGA font in CGA magenta and cyan (IBM PC 5150)

    python callouts.py OUT.png      # a contact sheet: every entry's caption, banner and a frame of its clip
"""
import json
import math
import os
import re
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))

def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


WORK = _work("")
FINAL = WORK + "callouts_final.json"
DEMOS = WORK + "demos/"
VGA = os.path.join(HERE, "pcfonts", "Px437_IBM_VGA_8x16.ttf")
CGA = os.path.join(HERE, "pcfonts", "Px437_IBM_CGA.ttf")
HEL = "/System/Library/Fonts/HelveticaNeue.ttc"

def entries():
    """[(title, group, platform, year, style, kind, standin, footage)] from the final list (callouts_final.json, in
    the callouts' order: screens pick by index). kind "clip": its footage; "parody": a parody title and group over a
    stand-in effect (plasma, tunnel, rotozoom, stars or rings), never footage. The style as the film has shown them:
    Amiga's copper bars, C64's light blue on blue, else the VGA font."""
    out = []
    if not os.path.exists(FINAL):
        return out
    for e in json.load(open(FINAL))["callouts"]:
        plat, _, year = e["platform_year"].rpartition(", ")
        style = "amiga" if "Amiga" in plat else "c64" if "C64" in plat else "pc90"
        out.append((e["title"], e["group"], plat, int(year), style, e["kind"], e.get("standin"),
                    WORK + e["footage"] if e["kind"] == "clip" else None, e.get("emblem")))
    return out

ENTRIES = entries()

def _font(path, size, index=0):
    from PIL import ImageFont
    return ImageFont.truetype(path, size, index=index)

# A horse's head (Future Glue's emblem), facing right: a bold silhouette, its outline in a unit square (x right, y down)
HORSE = [(0.0, 1.0), (0.05, 0.55), (0.20, 0.30), (0.30, 0.12), (0.34, 0.0), (0.42, 0.14), (0.47, 0.06), (0.52, 0.20),
         (0.75, 0.45), (0.95, 0.62), (1.0, 0.72), (0.96, 0.80), (0.82, 0.82), (0.65, 0.72), (0.50, 0.75), (0.45, 1.0)]
EMBLEM_W = 0.8                                            # its width over its height

def _emblem(d, name, x, y, hgt):
    """An emblem drawn into a mask (ink 255) at (x, y), hgt tall: the horse, with an eye."""
    if name != "horse":
        return
    wd = hgt * EMBLEM_W
    d.polygon([(x + px * wd, y + py * hgt) for px, py in HORSE], fill=255)
    r = max(0.6, hgt * 0.06)
    d.ellipse([x + 0.55 * wd - r, y + 0.36 * hgt - r, x + 0.55 * wd + r, y + 0.36 * hgt + r], fill=0)

def _text_img(lines, w, h, style, emblem=None):
    """A sign's face, w x h, in its period's style: RGB float 0..1 (lit), returned with its ink mask. emblem: (line,
    name), drawn before that line's words."""
    from PIL import Image, ImageDraw
    rgb = np.zeros((h, w, 3), np.float32)
    ink = Image.new("L", (w, h), 0)
    d = ImageDraw.Draw(ink)
    n = len(lines)
    if style in ("pc90", "c64", "cga"):
        path = VGA if style == "pc90" else CGA
        base = 16 if style == "pc90" else 8
        fit = max(1, min(int(h * 0.8 / n / base), int(w * 0.92 / (max(len(s) + (2 if emblem and emblem[0] == k else 0) for k, s in enumerate(lines)) * 8 * (2 if style != "pc90" else 1)) or 1)))
        size = base * fit
        f = _font(path, size)
    elif style == "pc00":
        f = _font(HEL, max(8, int(h * 0.62 / n)), 0)
        lines = [s.lower() for s in lines]
    else:
        f = _font(HEL, max(8, int(h * 0.7 / n)), 1)
    for i, s in enumerate(lines):
        tw = d.textlength(s, font=f)
        while tw > w * 0.94 and f.size > 6:
            f = _font(f.path, f.size - 1, getattr(f, "index", 0)) if hasattr(f, "path") else f
            tw = d.textlength(s, font=f)
        y = int(h * (i + 0.5) / n - f.size * 0.55)
        if emblem and emblem[0] == i:
            eh = h / n * 0.92
            ew = eh * EMBLEM_W
            x0 = (w - (tw + ew * 1.25)) / 2
            _emblem(d, emblem[1], x0, h * (i + 0.5) / n - eh / 2, eh)
            d.text((x0 + ew * 1.25, y), s, font=f, fill=255)
        else:
            d.text(((w - tw) / 2, y), s, font=f, fill=255)
    m = np.array(ink, np.float32) / 255
    yy = np.linspace(0, 1, h)[:, None]
    if style == "pc90":
        bg = np.array([0.0, 0.0, 0.55]); fg = np.array([1.0, 1.0, 0.33]) * (yy < 0.5) + np.array([0.33, 1.0, 1.0]) * (yy >= 0.5)
    elif style == "c64":
        bg = np.array([0.21, 0.16, 0.6]); fg = np.array([0.53, 0.48, 0.94]) * np.ones_like(yy)
    elif style == "cga":
        bg = np.array([0.0, 0.0, 0.0]); fg = np.array([1.0, 0.33, 1.0]) * (yy < 0.5) + np.array([0.33, 1.0, 1.0]) * (yy >= 0.5)
    elif style == "pc00":
        bg = np.array([0.02, 0.02, 0.02]); fg = np.array([0.95, 0.95, 0.95]) * np.ones_like(yy)
        rule = (np.abs(yy - 0.06) < 0.012) | (np.abs(yy - 0.94) < 0.012)
        m = np.maximum(m, rule * np.ones((1, w)) * 0.6)
    else:                                                 # amiga: copper bars through the letters
        bg = np.array([0.0, 0.0, 0.0])
        hue = yy * 12.0
        fg = np.concatenate([0.5 + 0.5 * np.sin(hue), 0.5 + 0.5 * np.sin(hue + 2.1), 0.5 + 0.5 * np.sin(hue + 4.2)], 1)
        fg = fg ** 0.7
    fgi = np.broadcast_to(fg[:, None, :] if fg.ndim == 2 else fg, (h, w, 3))
    rgb = bg * (1 - m[..., None]) + fgi * m[..., None]
    return rgb.astype(np.float32), m

_CAP, _BAN = {}, {}

def caption(i, w=160, h=30):
    """The lower third over a screen: title and group in the entry's style."""
    k = (i, w, h)
    if k not in _CAP:
        t, g, _, _, st = ENTRIES[i % len(ENTRIES)][:5]
        em = ENTRIES[i % len(ENTRIES)][8]
        _CAP[k] = _text_img([t, g], w, h, st, (1, em) if em else None)
    return _CAP[k]

def banner(i, w=256, h=96):
    """A sign or banner: the title big, the group under it (or the logo if the gathering found one)."""
    k = (i, w, h)
    if k not in _BAN:
        t, g, _, _, st = ENTRIES[i % len(ENTRIES)][:5]
        logo = _logo(i)
        if logo is not None:
            from PIL import Image
            im = Image.fromarray((np.clip(logo, 0, 1) * 255).astype(np.uint8)).convert("RGB")
            im.thumbnail((w, int(h * 0.72)))
            rgb = np.zeros((h, w, 3), np.float32)
            a = np.array(im, np.float32) / 255
            y0, x0 = 0, (w - a.shape[1]) // 2
            rgb[y0:y0 + a.shape[0], x0:x0 + a.shape[1]] = a
            em = ENTRIES[i % len(ENTRIES)][8]
            sub, m = _text_img([g], w, h - int(h * 0.72), st, (0, em) if em else None)
            rgb[int(h * 0.72):] = sub
            _BAN[k] = (rgb, np.ones((h, w), np.float32))
        else:
            em = ENTRIES[i % len(ENTRIES)][8]
            _BAN[k] = _text_img([t, g], w, h, st, (1, em) if em else None)
    return _BAN[k]

# ---- footage (the clips), or stand-ins (the parodies) ------------------------------------------------------------

_CLIPS = {}

def _frames(i):
    """A clip's 60 frames (120 x 160 RGB 0..1), or None for a parody."""
    if i not in _CLIPS:
        p = ENTRIES[i % len(ENTRIES)][7]
        if p and os.path.exists(p) and os.path.getsize(p) >= 160 * 120 * 3:
            a = np.fromfile(p, np.uint8)
            n = len(a) // (160 * 120 * 3)
            _CLIPS[i] = a[:n * 160 * 120 * 3].reshape(n, 120, 160, 3).astype(np.float32) / 255
        else:
            _CLIPS[i] = None
    return _CLIPS[i]

def _logo(i):
    """A clip's logo (only the demos we may show; never a parody's)."""
    e = ENTRIES[i % len(ENTRIES)]
    p = None if e[5] != "clip" else os.path.join(os.path.dirname(e[7]), "logo.png")
    if p and os.path.exists(p):
        from PIL import Image
        return np.array(Image.open(p).convert("RGB"), np.float32) / 255
    return None

def has_footage(i):
    return _frames(i) is not None

# The stand-ins, as CARDIAC works them out (segments/night/nmat.rs, standin()): an 80 x 60 picture a frame, each
# cell shown 2 x 2 on the 160 x 120 screen; whole numbers and printed tables; 60 frames round (k = f mod 60), each
# parody its own colours (an offset of 137 i into its palette).
SI_KINDS = ["plasma", "tunnel", "rotozoom", "stars", "rings"]
SI_W, SI_H, SI_STARS = 80, 60, 200

def _pal(cols, n=1000):
    """A palette round 1000 places through the colours (0..255), each band blended into the next."""
    cols = np.array(cols, float)
    t = np.arange(n) / n * len(cols)
    i = np.floor(t).astype(int)
    f = (t - i)[:, None]
    f = f * f * (3 - 2 * f)
    return np.round(cols[i % len(cols)] * (1 - f) + cols[(i + 1) % len(cols)] * f).astype(int)

_SIT = {}

def standin_tables():
    """The stand-ins' printed tables (whole numbers)."""
    if _SIT:
        return _SIT
    T = _SIT
    i = np.arange(8000)
    T["si_sin"] = np.round(63.5 + 63.5 * np.sin(2 * np.pi * i / 1000)).astype(int)        # 0..127, round 1000
    pals = [[(20, 0, 60), (200, 20, 120), (255, 160, 40), (255, 255, 200), (40, 200, 220), (10, 40, 140)],   # plasma
            [(230, 80, 200), (25, 50, 150)],                                                                 # tunnel
            [(255, 215, 50), (40, 12, 100)],                                                                 # rotozoom
            [(255, 255, 255)],                                                                               # stars
            [(255, 40, 40), (255, 200, 0), (20, 220, 80), (0, 120, 255), (160, 40, 255), (10, 10, 30)]]     # rings
    T["si_pal"] = np.concatenate([np.tile(_pal(p), (6, 1)) for p in pals])                  # 5 x 6000 rows of RGB
    ys, xs = np.mgrid[0:SI_H, 0:SI_W].astype(float)
    dx, dy = xs - 39.5, ys - 29.5
    r = np.hypot(dx, dy)
    T["si_rad3"] = np.round(r * 12).astype(int)                                              # the plasma's radial wave
    T["si_rad4"] = np.round(r * 40).astype(int)
    T["si_ang"] = np.round((np.arctan2(dy, dx) / (2 * np.pi) % 1.0) * 1000).astype(int) % 1000   # thousandths of a turn
    T["si_tunu"] = np.minimum(np.round(6000 / np.maximum(r, 1.5)), 999).astype(int)          # the tunnel's depth
    T["si_tunb"] = np.round(np.clip(r / 32, 0.08, 1) * 100).astype(int)                       # its light by the radius
    u = np.arange(2000)
    T["si_cu"] = (u // 100) % 2                                                              # the tunnel's checks
    T["si_cv"] = (u // 125) % 2
    w = np.arange(160_001) - 80_000
    T["si_ck"] = np.floor_divide(w, 8000) % 2                                                # the rotozoom's
    T["si_brz"] = np.round(np.clip((950 - np.arange(1000)) / 500, 0, 1) * 255).astype(int)   # a star's light by depth
    rng = np.random.default_rng(64)
    T["si_stars"] = np.stack([rng.integers(-999, 1000, SI_STARS), rng.integers(-999, 1000, SI_STARS), rng.integers(0, 900, SI_STARS)], 1)
    return T

def standin_kind(i):
    """The entry's stand-in (0..4), or -1 for a clip."""
    e = ENTRIES[i % len(ENTRIES)]
    return -1 if e[5] == "clip" else SI_KINDS.index(e[6])

def standin_cells(i, k):
    """The stand-in's frame k (0..59): 60 x 80 cells, RGB 0..255 (whole numbers, as CARDIAC's)."""
    T = standin_tables()
    kind = standin_kind(i)
    k = int(k) % 60
    off = 137 * (i % len(ENTRIES))
    pal = T["si_pal"][kind * 6000:(kind + 1) * 6000]
    xs = np.arange(SI_W)[None, :]
    ys = np.arange(SI_H)[:, None]
    S = T["si_sin"]
    if kind == 0:                                         # plasma: four waves, the colours cycling
        a = S[12 * xs + 50 * k]
        b = S[17 * ys + 2950 - 50 * k]
        c = S[9 * (xs + ys) + 100 * k]
        d = S[T["si_rad3"] + 6000 - 100 * k]
        idx = a + b + c + d + (off + 50 * k) % 1000
        return pal[idx]
    if kind == 1:                                         # tunnel: checks on a tube rushing at us, darker at its heart
        pu, pv = (40 * k) % 1000, (off + 10 * k) % 1000
        chk = T["si_cu"][T["si_tunu"] + pu] + T["si_cv"][T["si_ang"] + pv]
        col = np.where((chk == 1)[..., None], pal[0], pal[500])
        return col * T["si_tunb"][..., None] // 100
    if kind == 2:                                         # rotozoom: a check turning and breathing
        a = 16_667 * k + 1000 * (off % 1000)
        s_ = int(round(999 * math.sin(2 * math.pi * a / 1e6)))
        c_ = int(round(999 * math.cos(2 * math.pi * a / 1e6)))
        zs = int(round(999 * math.sin(2 * math.pi * (33_333 * k) / 1e6)))
        ks = 600 + 300 * zs // 999
        ka, kb = ks * c_ // 999, ks * s_ // 999
        u = (xs - 40) * ka - (ys - 30) * kb + 80_000
        v = (xs - 40) * kb + (ys - 30) * ka + 80_000
        chk = T["si_ck"][u] + T["si_ck"][v]
        return np.where((chk == 1)[..., None], pal[0], pal[500])
    if kind == 3:                                         # stars flying at us
        out = np.zeros((SI_H, SI_W, 3), int)
        st = T["si_stars"]
        z = (st[:, 2] + 900 - 15 * k + 7 * (off % 100)) % 900 + 50
        px = 40 + np.floor_divide(st[:, 0] * 40, z)
        py = 30 + np.floor_divide(st[:, 1] * 30, z)
        br = T["si_brz"][z]
        for j in range(len(st)):
            if 0 <= px[j] < SI_W and 0 <= py[j] < SI_H:
                out[py[j], px[j]] = br[j]
        return out
    idx = T["si_rad4"] + 3000 - 50 * k + off % 1000       # rings running out from the middle
    return pal[idx]

def standin(i, f):
    """A stand-in clip frame (120 x 160 RGB 0..1): its cells 2 x 2."""
    c = standin_cells(i, f)
    return (c.repeat(2, 0).repeat(2, 1) / 255).astype(np.float32)

def clip_frame(i, f, with_caption=True):
    """Frame f (12 fps) of entry i's clip, 120 x 160 RGB 0..1, its caption over the bottom quarter."""
    fr = _frames(i)
    c = fr[int(f) % len(fr)].copy() if fr is not None else standin(i, f)
    if with_caption:
        cap, m = caption(i, 160, 30)
        c[90:] = c[90:] * 0.25 * (1 - m[..., None]) + cap * (0.4 + 0.6 * m[..., None])
    return c

_MQ = {}

def marquee(h=16):
    """Every entry in one long line, VGA font: 'TITLE - GROUP  *  TITLE - GROUP ...', a 0/1 mask h tall."""
    if h not in _MQ:
        from PIL import Image, ImageDraw
        # (an entry with an emblem: two places after its group, the emblem drawn there)
        s = "   *   ".join("%s - %s%s" % (e[0].upper(), e[1].upper(), "   " if e[8] else "") for e in ENTRIES) + "   *   "
        f = _font(VGA, 16)
        dd = ImageDraw.Draw(Image.new("L", (1, 1)))
        w = int(dd.textlength(s, font=f)) + 8
        im = Image.new("L", (w, 16), 0)
        di = ImageDraw.Draw(im)
        di.text((0, 0), s, font=f, fill=255)
        for e in ENTRIES:
            if e[8]:
                at = s.index("%s - %s" % (e[0].upper(), e[1].upper())) + len("%s - %s" % (e[0].upper(), e[1].upper()))
                _emblem(di, e[8], dd.textlength(s[:at], font=f) + 5, 1, 14)
        _MQ[h] = (np.array(im) > 127).astype(np.float32)
    return _MQ[h]

def main():
    from PIL import Image
    n = len(ENTRIES)
    rows = []
    for i in range(n):
        cap, _ = caption(i, 320, 60)
        ban, _ = banner(i, 320, 120)
        clip = clip_frame(i, 30).repeat(2, 0).repeat(2, 1)[:, :320]
        rows.append(np.concatenate([np.pad(cap, ((0, 180), (0, 0), (0, 0))), np.pad(ban, ((0, 120), (0, 0), (0, 0))),
                                    np.pad(clip, ((0, 0), (0, 0), (0, 0)))], 1))
    cols = 3
    grid = []
    for r in range(0, n, cols):
        row = rows[r:r + cols]
        row += [np.zeros_like(rows[0])] * (cols - len(row))
        grid.append(np.concatenate(row, 1))
    Image.fromarray((np.concatenate(grid, 0) * 255).astype(np.uint8)).save(sys.argv[1])
    print(n, "entries;", sum(has_footage(i) for i in range(n)), "with footage")

if __name__ == "__main__":
    main()
