"""Prototype: the cold open's turn. The camera pulls back from the Mac's screen (the one-bit tunnel, CARD ATTACK
over it) to show the Mac Classic; it spins round and turns into an IBM PC with its colour monitor; the camera dives
into the PC's screen, a 320x200 CGA vortex in four colours, its palette cycling and flicking between the CGA
palettes, and the brag: 320 x 200 / 4 COLORS. Outlined, like the RJ-45 and the mixers; centimetres.

    python macpc.py OUT.mp4          a preview at 1536x768, 12 fps
"""
import math
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

import mtv_aha as M
from mtv import trans, rot_y
import rj45 as R
import macdesk as D

W, H, FPS = 1536, 768, 12
FONT = "pcfonts/Px437_IBM_CGA.ttf"                        # VileR's IBM CGA 8x8 (CREDITS.md)

# ---- the two machines, as lists of boxes that turn one into the other -----------------------------------------------

PLATINUM, PLATINUM_D = (222, 216, 200), (196, 190, 174)  # the Mac Classic's case
IBM, IBM_D, DARK = (214, 208, 190), (150, 146, 136), (52, 52, 56)
RAINBOW = [(97, 187, 70), (253, 184, 39), (245, 130, 31), (224, 58, 62), (150, 61, 151), (0, 157, 220)]

def ring(cx, cy, z, w, h, hw, hh, d, rgb):
    """A bezel round a screen hw x hh, as four boxes: (centre, size, colour)."""
    return [((cx, cy + (h + hh) / 4, z), (w, (h - hh) / 2, d), rgb), ((cx, cy - (h + hh) / 4, z), (w, (h - hh) / 2, d), rgb),
            ((cx + (w + hw) / 4, cy, z), ((w - hw) / 2, hh, d), rgb), ((cx - (w + hw) / 4, cy, z), ((w - hw) / 2, hh, d), rgb)]

MAC_SCREEN = ((0.0, 22.6, -14.9), (18.0, 18.0 * 342 / 512))  # centre, width x height (512x342)
PC_SCREEN = ((0.0, 30.2, -17.6), (26.0, 19.5))            # 4:3

def mac():
    parts = [((0, 16.75, 0), (24.6, 33.5, 29.0), PLATINUM)]                     # the case
    parts += ring(0, 22.6, -14.75, 21.6, 15.6, 18.0, 12.0, 0.9, PLATINUM_D)      # the screen's bezel
    parts += [((3.2, 8.6, -14.6), (9.4, 0.7, 0.6), DARK)]                        # the floppy slot
    parts += [((0, 0.6, -13.4), (22.0, 1.2, 3.0), PLATINUM_D)]                   # the foot
    parts += [((-8.6, 7.6 - 0.25 * k, -14.55), (1.2, 0.25, 0.3), c) for k, c in enumerate(RAINBOW)]   # the badge
    return parts

def pc():
    parts = [((0, 30.2, 1.0), (38.0, 30.0, 36.0), IBM)]                          # the monitor's case
    parts += ring(0, 30.2, -17.45, 32.0, 25.0, 26.0, 19.5, 0.9, IBM_D)            # its bezel
    parts += [((11.0, 10.4, -20.6), (14.8, 4.6, 0.6), DARK)]                     # the top floppy drive
    parts += [((0, 7.2, 0), (50.0, 14.4, 41.0), IBM)]                            # the system unit
    parts += [((11.0, 4.6, -20.6), (14.8, 4.6, 0.6), DARK)]                      # the second drive
    parts += [((-15.0, 3.2 + 0.9 * k, -20.6), (12.0, 0.4, 0.3), IBM_D) for k in range(5)]  # the vents
    parts += [((-17.5, 11.6, -20.6), (5.0, 1.4, 0.3), (40, 70, 140))]            # the badge
    return parts

def morph(a, b, m):
    """Each box of a becomes the box of b at its place; the extra ones grow from nothing where they'll be."""
    out = []
    for i in range(max(len(a), len(b))):
        pa = a[i] if i < len(a) else (b[i][0], (0, 0, 0), b[i][2])
        pb = b[i] if i < len(b) else (a[i][0], (0, 0, 0), a[i][2])
        c = tuple(p + (q - p) * m for p, q in zip(pa[0], pb[0]))
        s = tuple(p + (q - p) * m for p, q in zip(pa[1], pb[1]))
        col = tuple(int(p + (q - p) * m) for p, q in zip(pa[2], pb[2]))
        out.append((c, s, col))
    return out

def screen_at(m):
    (ca, sa), (cb, sb) = MAC_SCREEN, PC_SCREEN
    return tuple(p + (q - p) * m for p, q in zip(ca, cb)), tuple(p + (q - p) * m for p, q in zip(sa, sb))

# ---- the CGA vortex -------------------------------------------------------------------------------------------------

CGA = {                                                   # background black, then the palette's three
    "pal1 high": [(0, 0, 0), (85, 255, 255), (255, 85, 255), (255, 255, 255)],
    "pal0 high": [(0, 0, 0), (85, 255, 85), (255, 85, 85), (255, 255, 85)],
    "mode5 high": [(0, 0, 0), (85, 255, 255), (255, 85, 85), (255, 255, 255)],
    "pal1": [(0, 0, 0), (0, 170, 170), (170, 0, 170), (170, 170, 170)],
    "pal0": [(0, 0, 0), (0, 170, 0), (170, 0, 0), (170, 85, 0)],
}
FLICKS = ["pal0 high", "pal1 high", "mode5 high", "pal0", "pal1", "pal0 high"]

cy, cx = np.mgrid[0:200, 0:320].astype(np.float32)
VX, VY = (cx - 159.5), (cy - 99.5) * 1.2                  # CGA's pixels are taller than wide
VR = np.hypot(VX, VY) + 1e-3
VA = np.arctan2(VY, VX) / (2 * np.pi)

def brag():
    f = ImageFont.truetype(FONT, 8)
    im = Image.new("L", (320, 200), 0)
    d = ImageDraw.Draw(im)
    for i, s in enumerate(["320 x 200", "4 COLORS"]):
        w = d.textlength(s, font=f)
        d.text(((320 - w) // 2, 92 + 12 * i), s, fill=255, font=f)
    ink = np.array(im) > 127
    big = np.zeros_like(ink)                              # twice the size: 16x16 letters, still on CGA's grid
    ys, xs = np.nonzero(ink)
    for y, x in zip(ys, xs):
        y2, x2 = 100 + (y - 100) * 2, 160 + (x - 160) * 2
        big[y2:y2 + 2, x2:x2 + 2] = True
    box = np.zeros_like(big)
    ys, xs = np.nonzero(big)
    box[ys.min() - 6:ys.max() + 7, xs.min() - 8:xs.max() + 9] = True
    by, bx = np.nonzero(box)                              # into the bottom-left corner
    dy, dx = (200 - 8) - by.max(), 8 - bx.min()
    return np.roll(np.roll(big, dy, 0), dx, 1), np.roll(np.roll(box, dy, 0), dx, 1)

BRAG = brag()

BAYER = (np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]) + 0.5) / 16
DITHER = np.tile(BAYER, (50, 80))

def vortex(f, show_brag):
    """The film's tunnel again, on the PC: the checkerboard twisting down it, the checks in colours 2 and 3, the far
    end through colour 1 to black by an ordered dither; the palette flicks every second."""
    t = f / FPS
    depth = 1500 / VR
    u = VA * 16 + 0.6 * np.sin(t * 0.9) + depth * 0.012 * np.sin(t * 0.6)
    v = depth / 3.5 + t * 2.2
    check = np.where((np.floor(u) + np.floor(v)) % 2 == 0, 3, 2)
    level = 2 * np.clip((VR - 6) / 110, 0, 1) ** 1.2
    idx = np.where(level > 1 + DITHER, check, np.where(level > DITHER, 1, 0))
    if show_brag:
        big, box = BRAG
        idx[box] = 0
        idx[big] = 3
    pal = np.array(CGA[FLICKS[(f // FPS) % len(FLICKS)]], np.uint8)
    return pal[idx]

def cga_full(img320):
    """The CGA picture on the film's frame at 4:3: 1024x768, centred, black at the sides."""
    out = np.zeros((H, W, 3), np.uint8)
    yi = (np.arange(768) * 200 // 768)
    xi = (np.arange(1024) * 320 // 1024)
    out[:, 256:1280] = img320[yi][:, xi]
    return out

# ---- the Mac's screen -----------------------------------------------------------------------------------------------

def mac_screen(f):
    """The cold open's last picture, still going: the chunky one-bit tunnel, CARD ATTACK over it."""
    k = 108 + f                                           # carry on from where the titles were
    S = {"tunnel": (k // 3) * 3 / D.FPS, "title": (2, 1.0)}
    img = D.tunnel(S["tunnel"])
    white, rim = D.TITLES[2]
    img[rim] = True
    img[white] = False
    return np.where(img[..., None], 0, 255).astype(np.uint8).repeat(3, 2)

# ---- the shot -------------------------------------------------------------------------------------------------------

HFOV = math.radians(40)
T = math.tan(HFOV / 2)

def ease(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)

def framing(screen_w, fill_px):
    """How far back the camera stands for a screen screen_w wide to fill fill_px of the frame."""
    return screen_w * (W / fill_px) / (2 * T)

N_OUT, N_SPIN, N_HOLD, N_IN, N_CGA = 24, 30, 6, 20, 48

def render(f):
    m, angle = 0.0, 0.0
    sc_c, sc_s = MAC_SCREEN
    centre = np.array([0, 18.0, 0])
    far = 125.0
    side = np.array([0.42, 0.22, -1.0]); side /= np.linalg.norm(side)
    front = np.array([0, 0, -1.0])
    if f < N_OUT:                                         # pull back from the Mac's screen
        u = ease(f / (N_OUT - 1))
        d0 = framing(sc_s[0], 1024)
        tgt = np.array(sc_c) + (centre - np.array(sc_c)) * u
        dist = d0 + (far - d0) * u
        dirn = front + (side - front) * u
    elif f < N_OUT + N_SPIN:                              # spin and turn into the PC
        u = (f - N_OUT) / (N_SPIN - 1)
        angle = 2 * math.pi * ease(u)
        m = ease((u - 0.2) / 0.6)
        tgt, dist, dirn = centre, far, side
    else:                                                 # the PC; then dive into its screen
        m = 1.0
        u = ease((f - N_OUT - N_SPIN - N_HOLD) / (N_IN - 1)) if f >= N_OUT + N_SPIN + N_HOLD else 0.0
        pc_c, pc_s = PC_SCREEN
        d1 = framing(pc_s[0], 1024)
        tgt = centre + (np.array(pc_c) - centre) * u
        dist = far + (d1 - far) * u
        dirn = side + (front - side) * u
    dirn = dirn / np.linalg.norm(dirn)
    eye = tgt + dirn * dist

    s = M.Scene()
    spin = rot_y(angle)
    for c, size, col in morph(mac(), pc(), m):
        if min(size) > 0.01:
            s.box(spin @ trans(*c), size, col)
    c, size = screen_at(m)
    texs = {"screen": (mac_screen(f) if m < 0.5 else vortex(f, False)).astype(np.float32)}
    texs["screen"] = (texs["screen"], texs["screen"].mean(2), np.ones(texs["screen"].shape[:2], bool))
    s.quad(spin @ trans(*c), size[0], size[1], "screen")
    basis = M.look(eye, tgt)
    img, tone, oid, nrm, izb, tag = M.raster(s.tris, eye, basis, HFOV, W, H, texs)
    img = R.outline(img, oid, nrm, izb)
    return np.clip(img, 0, 255).astype(np.uint8)

def main():
    out = sys.argv[1]
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}",
                           "-r", str(FPS), "-i", "-", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "16",
                           "-movflags", "+faststart", out], stdin=subprocess.PIPE)
    n3d = N_OUT + N_SPIN + N_HOLD + N_IN
    for f in range(n3d + N_CGA):
        frame = render(f) if f < n3d else cga_full(vortex(f, show_brag=f >= n3d + 6))
        ff.stdin.write(frame.tobytes())
    ff.stdin.close()
    ff.wait()
    print(n3d + N_CGA, "frames,", round((n3d + N_CGA) / FPS, 1), "s")

if __name__ == "__main__":
    main()
