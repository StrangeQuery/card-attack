"""The screening room: the dorm's trick again between the drive and the dive, a little fancier:
halfway through the accelerated drive (`accel`'s 768x384 half) the camera pulls back out of the picture into a small
hall where it's projected: a 2:1 screen on the front wall, the projector on a cart among them with its beam
through the haze over their heads, a loose crowd standing, from behind
(static silhouettes); the dive (`deep`) starts on the screen while we're still out, and we move back in. A flat
picture (drawn once, in perspective), lit by the screen, as prototypes/dorm.py.

    python prototypes/screening.py preview OUT.mp4    # the animatic over the cut's own frames (the full cut's preview)

The hall in metres, projected once (bigger, more people, standing, the projector on a cart): the eye at the
back 2.4 m up on a riser, the front wall 15 m off, 12 m wide, 5.3 m high; the screen 5 x 2.5 m, its bottom 2.0 m up
(f 1560: the screen 520 x 260 in the frame); 130 people standing loose; the cart in the aisle, 6.5 m off. Layers as the dorm's, and the
beam: its share of the haze's light (falling off from the lens), lit by the screen's mean colour.
"""
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dorm import lin, srgb, ease  # noqa: E402


def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


W, H = 1536, 768
ART = 2
SS = 2
F, HZ, EY = 1560.0, 345.0, 2.4     # on a riser at the back, over their heads (none covers the screen)
AMB = np.array([0.035, 0.035, 0.055])
GAIN = 4.0
RIM = 0.9
BEAM = 2.2
HAZE = 1.6                  # the people's tone from the haze between them and us, lit by the screen
CUT_PREVIEW = ("previews", "fullcut_cardiac_2026-10-27a.mp4")   # in $CARD_ATTACK_WORK
ACCEL_HD, DEEP = 1665, 1783          # the cut's frames: the accelerated drive's first, the dive's first


def P(X, Y, d):
    return (W / 2 + F * X / d, HZ - F * (Y - EY) / d)


FD = 15.0                             # the front wall
_a, _b = P(-2.5, 4.5, FD), P(2.5, 2.0, FD)
SCREEN = (_a[0], _a[1], _b[0], _b[1])
LENS = (-0.2, 1.45, 6.5)              # the projector's lens, on a cart in the open space near us


RENDERS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "segments/rooms/data/dorm3d")
S_MON, S_BEAM, S_CONST = 3.5, 8.0, 4.0      # the renders' light to the layers' (set against the flat drawing's brightness)


def draw_art():
    """The layers from the 3D renders (prototypes/dorm3d/hall.py, the whole frame at ART): albedo the diffuse colour;
    glow the screen's light over it (white light 1 through GAIN); the beam the projector's light (its haze and what it
    catches, through BEAM); emissive the rest (the EXIT signs, the party's monitors, the lens); the screen where the
    picture's object shows, its whole rectangle as drawn."""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "dorm3d"))
    import exr
    m, b, c, k = (exr.read(os.path.join(RENDERS, f"hall_{x}.exr")) for x in ("mon", "beam", "const", "mask"))
    alb = np.clip(m["Diffuse Color"][..., :3].astype(float), 0, 1)
    asum = alb.sum(-1)
    mon = m["Combined"][..., :3].sum(-1) * S_MON
    glow = np.where(asum > 0.003, mon / np.maximum(asum, 1e-6) / GAIN, 0.0)
    glow = np.minimum(glow, 1.5 / np.maximum(alb.max(-1), 1e-3) / GAIN)
    h, w = alb.shape[:2]
    rect = Image.new("L", (w, h), 0)
    ImageDraw.Draw(rect).rectangle([round(v * ART) for v in SCREEN], fill=255)
    return {"alb": srgb(alb), "glow": glow, "emi": srgb(c["Combined"][..., :3] * S_CONST / 0.9),
            "beam": b["Combined"][..., :3].mean(-1) * S_BEAM / BEAM,
            "screen": (np.abs(k["Object Index.X"].reshape(h, w) - 1) < 0.5).astype(float),
            "rim": np.zeros((h, w)), "haze": np.zeros((h, w)), "rect": np.asarray(rect, float) / 255}


def draw_art_flat():
    """The flat drawing (superseded by the 3D renders)."""
    K = ART * SS
    w, h = W * K, H * K
    def k(*v):
        return [round(x * K) for x in v]
    def q(*pts):
        return k(*[c for p in pts for c in P(*p)])
    alb = Image.new("RGB", (w, h), (0, 0, 0)); glw = Image.new("L", (w, h), 0)
    emi = Image.new("RGB", (w, h), (0, 0, 0)); people = Image.new("L", (w, h), 0)
    beam = Image.new("L", (w, h), 0)
    haze = Image.new("L", (w, h), 0)
    dh = ImageDraw.Draw(haze)
    tops = Image.new("L", (w, h), 0)
    dt = ImageDraw.Draw(tops)
    da, dg, de, dp, db = (ImageDraw.Draw(x) for x in (alb, glw, emi, people, beam))
    def face(pts, col, g):
        da.polygon(q(*pts), fill=col); dg.polygon(q(*pts), fill=g)
    L_, R_, CEIL, ND = -6.0, 6.0, 4.5, 1.2
    # the hall: ceiling, side walls, floor, the front wall; the screen's frame
    face([(L_, CEIL, ND), (R_, CEIL, ND), (R_, CEIL, FD), (L_, CEIL, FD)], (40, 40, 46), 120)
    face([(L_, CEIL, ND), (L_, CEIL, FD), (L_, 0, FD), (L_, 0, ND)], (70, 66, 74), 170)
    face([(R_, CEIL, ND), (R_, CEIL, FD), (R_, 0, FD), (R_, 0, ND)], (70, 66, 74), 170)
    face([(L_, 0, ND), (R_, 0, ND), (R_, 0, FD), (L_, 0, FD)], (40, 34, 36), 110)
    face([(L_, CEIL, FD), (R_, CEIL, FD), (R_, 0, FD), (L_, 0, FD)], (80, 78, 90), 255)
    face([(-2.6, 3.9, FD), (2.6, 3.9, FD), (2.6, 1.2, FD), (-2.6, 1.2, FD)], (24, 24, 26), 255)
    # the ceiling's beams, a door each side with its EXIT sign, speakers stacked each side of the screen, a banner
    # along the left wall
    for d in (4.0, 7.0, 10.0, 13.0):
        face([(L_, CEIL, d), (R_, CEIL, d), (R_, CEIL - 0.3, d), (L_, CEIL - 0.3, d)], (34, 34, 38), 110)
    for X in (L_, R_):
        face([(X, 2.2, 11.0), (X, 2.2, 12.2), (X, 0, 12.2), (X, 0, 11.0)], (50, 44, 40), 150)
        ex = [P(X, 2.6, 11.2), P(X, 2.6, 12.0), P(X, 2.35, 12.0), P(X, 2.35, 11.2)]
        de.polygon(k(*[c for p in ex for c in p]), fill=(30, 200, 70))
    for x0 in (-4.2, 3.3):
        face([(x0, 2.6, FD - 0.4), (x0 + 0.9, 2.6, FD - 0.4), (x0 + 0.9, 0.0, FD - 0.4), (x0, 0.0, FD - 0.4)], (28, 28, 32), 200)
    face([(L_, 4.0, 4.0), (L_, 4.0, 12.0), (L_, 3.4, 12.0), (L_, 3.4, 4.0)], (40, 30, 90), 180)
    lx, ly = P(*LENS); r = F * 0.012 / LENS[2]
    # its beam: from the lens to the screen's corners, through the haze
    pts = [P(*LENS), (SCREEN[0], SCREEN[1]), (SCREEN[2], SCREEN[1]), (SCREEN[2], SCREEN[3]), (SCREEN[0], SCREEN[3])]
    for tri in ((0, 1, 2), (0, 2, 3), (0, 3, 4), (0, 4, 1)):
        db.polygon(k(*[c for i in tri for c in pts[i]]), fill=255)
    # the audience: a loose crowd standing (not rows of chairs; more authentic), clear round the cart and of the
    # line from us to it (let some of them cover the screen); far ones first
    rng = np.random.default_rng(1995)
    crowd = []
    while len(crowd) < 130:
        X, d = rng.uniform(-5.6, 5.6), rng.uniform(2.3, 13.8)
        if d < LENS[2] and abs(X - LENS[0] * d / LENS[2]) < 0.4:
            continue
        if math.hypot(X - LENS[0], d - LENS[2]) < 0.7:
            continue
        if any(math.hypot(X - x, d - e) < 0.5 for x, e, *_ in crowd):
            continue
        crowd.append((X, d, 1.58 + rng.uniform(-0.1, 0.14), 0.1 + rng.uniform(-0.008, 0.014),
                      rng.choice(["short", "long", "cap", "curly", "short", "bun"])))
    cart_drawn = False
    for X, d, hy, r_, hair in sorted(crowd, key=lambda c: -c[1]):
        if not cart_drawn and d < LENS[2] - 0.35:
            cart_drawn = True
            def cface(pts, col, g):
                face(pts, col, g); dp.polygon(q(*pts), fill=0); dh.polygon(q(*pts), fill=0)
            # the projector on its cart (the people behind it hidden where it stands): the cart (its top, its shelf, its
            # legs), the projector
            cz, cxm = LENS[2] - 0.35, LENS[0]
            CT = 1.22                           # the cart's top
            cface([(cxm + -0.35, CT, cz - 0.3), (cxm + 0.35, CT, cz - 0.3), (cxm + 0.35, CT, cz + 0.3), (cxm + -0.35, CT, cz + 0.3)], (60, 60, 66), 60)
            cface([(cxm + -0.35, CT, cz - 0.3), (cxm + 0.35, CT, cz - 0.3), (cxm + 0.35, CT - 0.06, cz - 0.3), (cxm + -0.35, CT - 0.06, cz - 0.3)], (40, 40, 44), 40)
            cface([(cxm + -0.35, 0.45, cz - 0.3), (cxm + 0.35, 0.45, cz - 0.3), (cxm + 0.35, 0.40, cz - 0.3), (cxm + -0.35, 0.40, cz - 0.3)], (40, 40, 44), 40)
            for x in (-0.33, 0.30):
                cface([(cxm + x, CT - 0.06, cz - 0.3), (cxm + x + 0.03, CT - 0.06, cz - 0.3), (cxm + x + 0.03, 0.0, cz - 0.3), (cxm + x, 0.0, cz - 0.3)], (36, 36, 40), 40)
            cface([(cxm + -0.2, CT + 0.32, cz - 0.15), (cxm + 0.2, CT + 0.32, cz - 0.15), (cxm + 0.16, CT + 0.30, cz + 0.25), (cxm + -0.16, CT + 0.30, cz + 0.25)], (84, 84, 90), 60)
            cface([(cxm + -0.2, CT + 0.32, cz - 0.15), (cxm + 0.2, CT + 0.32, cz - 0.15), (cxm + 0.2, CT, cz - 0.15), (cxm + -0.2, CT, cz - 0.15)], (70, 70, 76), 60)
        args = (X, hy, d, r_, hair)
        person(dp, k, *args, sitting=False, fill=255)
        person(dh, k, *args, sitting=False, fill=round(255 * min(1.0, (d - 2.0) / 12.0) ** 1.3), top=True)
        person(dt, k, *args, sitting=False, fill=255, top=True)
    de.ellipse(k(lx - r, ly - r, lx + r, ly + r), fill=(255, 255, 240))
    pm = np.asarray(people, float) / 255
    blur = 3 * K
    soft = np.asarray(Image.fromarray((pm * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(blur)), float) / 255
    rim = np.clip((pm - soft) * 3.0, 0, 1) * pm
    gy = np.gradient(soft, axis=0) * blur * 4
    rim = rim * np.clip(0.3 + gy, 0, 1) * (np.asarray(tops, float) / 255)      # heads and shoulders only
    yy, xx = np.mgrid[0:h, 0:w] / K
    cx, cy = (SCREEN[0] + SCREEN[2]) / 2, (SCREEN[1] + SCREEN[3]) / 2
    fall = 1.0 / (1.0 + (((xx - cx) / 1.2) ** 2 + ((yy - cy) / 0.9) ** 2) / 420.0 ** 2)
    gl = np.asarray(glw, float) / 255 * fall * (1 - pm)
    # the beam's haze: brighter near the lens, softened at its edges; hidden behind the people
    lx, ly = P(*LENS)
    dl = np.hypot(xx - lx, yy - ly)
    bm = np.asarray(beam.filter(ImageFilter.GaussianBlur(4 * K)), float) / 255
    bm = bm * (0.25 + 0.75 / (1 + dl / 120.0)) * (1 - pm)
    sm = Image.new("L", (w, h), 0)
    ImageDraw.Draw(sm).rectangle(k(*SCREEN), fill=255)
    smv = np.asarray(sm, float) / 255 * (1 - pm)          # the people in front of the screen hide it
    bm = bm * (1 - smv)
    a = np.asarray(alb, float) * (1 - pm[..., None]) + np.array([5, 5, 7]) * pm[..., None]
    def down(x):
        im = Image.fromarray(np.clip(x, 0, 255).astype(np.uint8))
        return np.asarray(im.resize((W * ART, H * ART), Image.BOX), float) / 255
    return {"alb": down(a), "glow": down(gl * 255), "emi": down(np.asarray(emi, float) * (1 - pm)[..., None]),
            "screen": down(smv * 255), "rim": down(rim * 255), "beam": down(bm * 255),
            "haze": down(np.asarray(haze, float) * pm), "rect": down(np.asarray(sm, float))}


def person(dp, k, X, hy_m, d, r_, hair, sitting, fill=255, top=False):
    """A silhouette from behind: head, neck, shoulders (down past the frame's bottom)."""
    hx, hy = P(X, hy_m, d)
    r = F * r_ / d
    sw = F * 0.21 / d
    _, sy = P(X, hy_m - 0.13, d)
    if sitting:
        dp.rounded_rectangle(k(hx - sw, sy, hx + sw, sy + F * 0.9 / d), radius=round(F * 0.09 / d * ART * SS), fill=fill)
    elif top:
        # (only the head and shoulders: the haze's share)
        dp.rounded_rectangle(k(hx - sw, sy, hx + sw * 0.85, sy + F * 0.35 / d), radius=round(F * 0.09 / d * ART * SS), fill=fill)
    else:
        # the torso to the hips, then two legs to the floor
        dp.rounded_rectangle(k(hx - sw, sy, hx + sw * 0.85, sy + F * 0.62 / d), radius=round(F * 0.09 / d * ART * SS), fill=fill)
        _, fy = P(X, 0.0, d)
        for lx0 in (-0.16, 0.01):
            x0, _ = P(X + lx0, 0, d); x1, _ = P(X + lx0 + 0.15, 0, d)
            dp.rectangle(k(x0, sy + F * 0.5 / d, x1, fy), fill=fill)
    dp.rectangle(k(hx - r * 0.42, hy + r * 0.5, hx + r * 0.42, sy + 2), fill=fill)
    dp.ellipse(k(hx - r, hy - r, hx + r, hy + r), fill=fill)
    if hair == "long":
        dp.rounded_rectangle(k(hx - r * 1.04, hy - r * 0.2, hx + r * 1.04, hy + r * 2.0), radius=round(r * 0.5 * ART * SS), fill=fill)
    if hair == "cap":
        dp.ellipse(k(hx - r * 1.02, hy - r * 1.06, hx + r * 1.02, hy + r * 0.2), fill=fill)
        dp.rounded_rectangle(k(hx - r * 0.5, hy + r * 0.35, hx + r * 0.5, hy + r * 0.78), radius=round(r * 0.2 * ART * SS), fill=fill)
    if hair == "bun":
        dp.ellipse(k(hx - r * 0.45, hy - r * 1.45, hx + r * 0.45, hy - r * 0.55), fill=fill)
    if hair == "curly":
        for a in range(0, 360, 30):
            ax, ay = hx + math.cos(math.radians(a)) * r * 0.85, hy + math.sin(math.radians(a)) * r * 0.85
            dp.ellipse(k(ax - r * 0.42, ay - r * 0.42, ax + r * 0.42, ay + r * 0.42), fill=fill)


# ---- the drive pops out to the desktop: a Windows 95 desktop (no logos, no trademarks), a command prompt open, `seaplusplus` typed,
# an ASCII banner and LOADING as we zoom in; the dive starts as we get there. Drawn at 640 x 480, shown 1.6x and
# pillarboxed on the 2:1 screen.
VGA = os.path.join(os.path.dirname(os.path.abspath(__file__)), "pcfonts", "Px437_IBM_VGA_8x16.ttf")
_GLYPHS = {
    "U": ["#   #", "#   #", "#   #", "#   #", " ### "], "N": ["#   #", "##  #", "# # #", "#  ##", "#   #"],
    "D": ["#### ", "#   #", "#   #", "#   #", "#### "], "E": ["#####", "#    ", "#### ", "#    ", "#####"],
    "R": ["#### ", "#   #", "#### ", "#  # ", "#   #"], "W": ["#   #", "#   #", "# # #", "## ##", "#   #"],
    "A": [" ### ", "#   #", "#####", "#   #", "#   #"], "T": ["#####", "  #  ", "  #  ", "  #  ", "  #  "],
    "S": [" ####", "#    ", " ### ", "    #", "#### "], "+": ["     ", "  #  ", " ### ", "  #  ", "     "]}
# the banner (the game's a nod, its maker spoofed as Unseen Planets, an Easter egg in small type): SEA++ in
# solid blocks two characters wide a pixel (CP437's full block), a water line of shades under it
BANNER = ["Unseen Planets presents", ""] + \
         ["".join(_GLYPHS[c][r].replace("#", "\u2588\u2588").replace(" ", "  ") + "  " for c in "SEA++") for r in range(5)] + \
         ["", ("\u2591\u2592\u2593\u2592" * 15)]
TYPED = "seaplusplus"
TYPE0 = 24                  # the prompt waits 2 s before the typing
LOAD_N = 60                 # LOADING takes 5 s


def desktop_base():
    """The desktop with nothing in the prompt's text area (640 x 480 RGB bytes): the icons, the taskbar, the window."""
    im = Image.new("RGB", (640, 480), (0, 128, 128))
    d = ImageDraw.Draw(im)
    d.fontmode = "1"
    sans = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 11)
    sansb = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 11)
    def bevel(x0, y0, x1, y1, down=False):
        d.rectangle([x0, y0, x1, y1], fill=(192, 192, 192))
        hi, lo = ((128, 128, 128), (255, 255, 255)) if down else ((255, 255, 255), (64, 64, 64))
        d.line([(x0, y1), (x0, y0), (x1, y0)], fill=hi); d.line([(x0, y1), (x1, y1), (x1, y0)], fill=lo)
    # the desktop's icons (plain shapes), their names
    for i, name in enumerate(("My Computer", "DEMOS", "Recycle Bin")):
        y = 12 + i * 70
        d.rectangle([22, y, 54, y + 28], fill=(200, 200, 200), outline=(0, 0, 0))
        d.rectangle([26, y + 4, 50, y + 20], fill=(0, 0, 128))
        w = d.textlength(name, font=sans)
        d.text((38 - w / 2, y + 34), name, font=sans, fill=(255, 255, 255))
    # the taskbar, Start, the prompt's button, the clock
    d.rectangle([0, 452, 639, 479], fill=(192, 192, 192)); d.line([(0, 453), (639, 453)], fill=(255, 255, 255))
    bevel(3, 456, 56, 476); d.text((14, 459), "Start", font=sansb, fill=(0, 0, 0))
    bevel(62, 456, 222, 476, down=True); d.text((72, 459), "Command Prompt", font=sansb, fill=(0, 0, 0))
    bevel(570, 456, 636, 476, down=True); d.text((581, 459), "11:42 PM", font=sans, fill=(0, 0, 0))
    # the prompt's window: its frame, the title bar, its buttons, the text area
    X0, Y0, X1, Y1 = 84, 26, 624, 430
    bevel(X0, Y0, X1, Y1)
    d.rectangle([X0 + 3, Y0 + 3, X1 - 3, Y0 + 20], fill=(0, 0, 128))
    d.text((X0 + 8, Y0 + 5), "Command Prompt", font=sansb, fill=(255, 255, 255))
    for j in range(3):
        bevel(X1 - 56 + j * 17, Y0 + 5, X1 - 42 + j * 17, Y0 + 18)
    d.text((X1 - 22, Y0 + 3), "x", font=sansb, fill=(0, 0, 0))
    tx0, ty0 = X0 + 5, Y0 + 24
    d.rectangle([tx0, ty0, X1 - 5, Y1 - 5], fill=(0, 0, 0))
    return np.asarray(im)


TEXT_AT = (91, 52)           # the text area's first cell (the prompt's window at 84, 26: its frame, its title bar)


def desktop(f):
    """The desktop f frames after the drive closed: the prompt, its cursor blinking; typing from TYPE0 (a character a
    frame), Enter; the banner from type_end + 2, LOADING's dots and its bar filling over LOAD_N frames."""
    im = Image.fromarray(desktop_base())
    d = ImageDraw.Draw(im)
    d.fontmode = "1"
    vga = ImageFont.truetype(VGA, 16)
    tx0, ty0 = TEXT_AT[0] - 2, TEXT_AT[1] - 2
    lines = ["C:\\DEMOS>drive.exe", "",
             "C:\\DEMOS>" + TYPED[:max(0, min(len(TYPED), f - TYPE0))]]
    type_end = TYPE0 + len(TYPED)
    if f >= type_end + 2:
        lines += [""] + BANNER + [""]
        dots = "." * ((f - type_end) // 4 % 4)
        n = min(30, (f - type_end - 2) * 30 // LOAD_N)
        lines += ["   ><(((o>   ~   ~   ~   LOADING" + dots, "", "   [" + "#" * n + " " * (30 - n) + "]"]
    for i, l in enumerate(lines[-24:]):
        d.text((tx0 + 2, ty0 + 2 + i * 16), l, font=vga, fill=(192, 192, 192))
    # the cursor: under the end of the last line, blinking
    if f < type_end + 2 and (f // 3) % 2 == 0:
        cx = tx0 + 2 + 8 * len(lines[-1])
        cy = ty0 + 2 + (len(lines[-24:]) - 1) * 16
        d.rectangle([cx, cy + 13, cx + 7, cy + 14], fill=(192, 192, 192))
    big = im.resize((1024, 768), Image.NEAREST)
    out = Image.new("RGB", (W, H), (0, 0, 0)); out.paste(big, (256, 0))
    return np.asarray(out, float) / 255


def the_end(f):
    """THE END (the drive needs a better ending; fade it so it feels finished): on black after the drive's fade,
    the words fading in over 6 frames."""
    im = Image.new("RGB", (W, H), (0, 0, 0))
    d = ImageDraw.Draw(im)
    fnt = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 150)
    t = "THE END"
    w = d.textlength(t, font=fnt)
    a = round(255 * min(1.0, max(0.0, (f + 1) / 6)))
    d.text(((W - w) / 2, 290), t, font=fnt, fill=(a, a, a))
    return np.asarray(im, float) / 255

def room_lit(L, mean, fade):
    light = AMB + L["glow"][..., None] * mean * GAIN
    return (lin(L["alb"]) * light + lin(L["emi"]) * 0.9 + L["rim"][..., None] * mean * RIM
            + L["beam"][..., None] * mean * BEAM + L["haze"][..., None] * mean * HAZE * 0.05) * fade


def frame(L, room, picture, s, centre):
    yy, xx = np.mgrid[0:H, 0:W].astype(float)
    ux = centre[0] + (xx - W / 2) / s
    uy = centre[1] + (yy - H / 2) / s
    ax = np.clip((ux * ART).astype(int), 0, W * ART - 1)
    ay = np.clip((uy * ART).astype(int), 0, H * ART - 1)
    col = srgb(room[ay, ax])
    m = L["screen"][ay, ax][..., None]
    ph, pw = picture.shape[:2]
    px = np.clip(((ux - SCREEN[0]) / (SCREEN[2] - SCREEN[0]) * pw).astype(int), 0, pw - 1)
    py = np.clip(((uy - SCREEN[1]) / (SCREEN[3] - SCREEN[1]) * ph).astype(int), 0, ph - 1)
    return col * (1 - m) + picture[py, px] * m


def cut_frames(first, n):
    """The full cut's preview frames (1024 x 512), up to 1536 x 768."""
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", _work(*CUT_PREVIEW), "-vf",
                          f"select='between(n\\,{first}\\,{first + n - 1})',scale=1536:768", "-fps_mode", "passthrough",
                          "-f", "rawvideo", "-pix_fmt", "rgb24", "-"], capture_output=True).stdout
    return np.frombuffer(raw, np.uint8).reshape(-1, H, W, 3)


def preview(out):
    L = draw_art()
    drive = cut_frames(ACCEL_HD + 30, 88)       # the drive from its 30th frame to its end
    dive = cut_frames(DEEP, 100)
    cx, cy = (SCREEN[0] + SCREEN[2]) / 2, (SCREEN[1] + SCREEN[3]) / 2
    s0 = W / (SCREEN[2] - SCREEN[0])            # the screen fills the frame: the picture exactly
    PULL, BACK = 29, 26                         # (frames of the drive shown here: 30.., so 29 is its 59th, halfway)
    p = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "768x384",
                          "-r", "12", "-i", "-", "-c:v", "libx264", "-crf", "20", "-pix_fmt", "yuv420p",
                          "-movflags", "+faststart", out], stdin=subprocess.PIPE)
    def emit(img):
        a = (np.clip(img, 0, 1) * 255).astype(np.uint16)
        a = ((a[0::2, 0::2] + a[1::2, 0::2] + a[0::2, 1::2] + a[1::2, 1::2]) // 4).astype(np.uint8)
        p.stdin.write(a.tobytes())
    def zoom(u):
        sc = math.exp(math.log(s0) * (1 - u))
        w_ = (sc - 1) / (s0 - 1)
        return sc, (cx * w_ + W / 2 * (1 - w_), cy * w_ + H / 2 * (1 - w_))
    still = None
    FADE = 12                                   # the drive's last second fades to black
    for f in range(len(drive)):
        pic = drive[f] / 255.0 * min(1.0, (len(drive) - 1 - f) / FADE)
        if f < PULL:
            emit(pic); continue
        s, c = zoom(ease((f - PULL) / BACK))
        mean = lin(pic[::8, ::8]).reshape(-1, 3).mean(0)
        img = frame(L, room_lit(L, mean, min(1.0, (f - PULL) / 8)), pic, s, c)
        if f == 70: still = img
        emit(img)
    # THE END on black after the drive's fade, 2 s; the desktop; from the typing a slow zoom in (ZIN); the dive from the end of
    # LOADING, a second and a half before we get there
    fixed_s, fixed_c = zoom(1.0)
    for f in range(24):
        pic = the_end(f)
        mean = lin(pic[::8, ::8]).reshape(-1, 3).mean(0)
        emit(frame(L, room_lit(L, mean, 1.0), pic, fixed_s, fixed_c))
    # the zoom in from LOADING's start, slow at first (the dive starts while we're still in the room, and we
    # close in on it playing): about 9 s, the dive from LOADING's end with the screen not much past half the frame
    z0 = TYPE0 + len(TYPED) + 2
    dive_at = z0 + LOAD_N
    ZIN = 108
    for f in range(z0 + ZIN):
        pic = desktop(f) if f < dive_at else dive[f - dive_at] / 255.0
        u = ease((f - z0) / ZIN) ** 1.5 if f >= z0 else 0.0
        s, c = zoom(1 - u)
        mean = lin((pic[::8, 256:1280:8] if f < dive_at else pic[::8, ::8])).reshape(-1, 3).mean(0)
        emit(frame(L, room_lit(L, mean, 1 - u), pic, s, c))
    for f in range(z0 + ZIN - dive_at, z0 + ZIN - dive_at + 24):
        emit(dive[f] / 255.0)
    p.stdin.close(); p.wait()
    Image.fromarray((np.clip(still, 0, 1) * 255).astype(np.uint8)).save(out.replace(".mp4", "_still.png"))


if __name__ == "__main__":
    if sys.argv[1] == "preview":
        preview(sys.argv[2])
