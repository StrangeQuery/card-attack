"""The film's last seconds: with the CARDIAC on the giant screen, a second before the cut a
hard cut (no stutter: it can't work at 12 fps) to a big, fat SQ in a sharp 3D look like HBO's 1980s logo, and "Strange Query" materialising underneath it.

    python prototypes/ending_sq.py preview OUT.mp4    # the last 6 s of the cut's preview, the stutter, the SQ

The SQ: classic chrome lettering, drawn once as a flat picture: the letters' cover, how far into each letter a pixel
is (its bevel), and the reflection's offset (the bevel bends what it reflects); the chrome itself a printed gradient
(sky above the horizon line, earth below) indexed by the row plus the offset, a dark outline, a star glint sweeping
across. "Strange Query" materialises: its pixels come in by a random order over a second, each as a white sparkle
first.
"""
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont


def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
W, H = 1536, 768
CUT_PREVIEW = ("previews", "fullcut_cardiac_2026-10-27c.mp4")   # in $CARD_ATTACK_WORK
STADIUM_AT = 3418 - 575          # the cut's frame of the stadium's first
BLACK_AT = 539                   # the stadium's first black frame
FONT = "/System/Library/Fonts/Supplemental/Arial Black.ttf"


def letters_flat(S=2):
    """The SQ's front face in its own plane (emulate HBO's wordmark, bolder and fatter): heavy upright letters
    packed tight, the S butting into the Q; the Q a fat ring, its counter open, and a short heavy tail. 2x the frame's size; and the seam
    where the S lies over the Q (dark on the faces)."""
    im = Image.new("L", (W * S, H * S), 0)
    d = ImageDraw.Draw(im)
    top, bot = 150, 560
    h = bot - top
    qx0, qx1 = 770, 770 + int(h * 1.12)
    d.ellipse([qx0 * S, top * S, qx1 * S, bot * S], fill=255)
    # its counter open: a fat ring, the hole a little right of the middle as the HBO O's
    hx, hy, hrx, hry = (qx0 + qx1) / 2 + h * 0.03, (top + bot) / 2, (qx1 - qx0) * 0.23, h * 0.25
    d.ellipse([(hx - hrx) * S, (hy - hry) * S, (hx + hrx) * S, (hy + hry) * S], fill=0)
    tx, ty = qx0 + (qx1 - qx0) * 0.80, top + h * 0.80
    tw = h * 0.22
    ang = math.radians(38)
    dx, dy = math.cos(ang), math.sin(ang)
    nx, ny = -dy, dx
    L0, L1 = -h * 0.10, h * 0.30
    pts = [(tx + dx * L0 + nx * tw / 2, ty + dy * L0 + ny * tw / 2), (tx + dx * L1 + nx * tw / 2, ty + dy * L1 + ny * tw / 2),
           (tx + dx * L1 - nx * tw / 2, ty + dy * L1 - ny * tw / 2), (tx + dx * L0 - nx * tw / 2, ty + dy * L0 - ny * tw / 2)]
    d.polygon([(x * S, y * S) for x, y in pts], fill=255)
    f = ImageFont.truetype(FONT, 600 * S)
    g = Image.new("L", (900 * S, 900 * S), 0)
    ImageDraw.Draw(g).text((50 * S, 0), "S", font=f, fill=255)
    g = g.crop(g.getbbox())
    gh = h * S
    gw = int(g.width * gh / g.height * 1.05)
    g = g.resize((gw, gh), Image.LANCZOS)
    # fatter: the S's strokes thickened
    g = g.filter(ImageFilter.MaxFilter(2 * (9 * S // 2) + 1))
    sx = qx0 * S - gw + int(h * 0.07 * S)
    im.paste(Image.new("L", g.size, 255), (sx, top * S), g)
    # (the bottom a straight line: everything sits on the baseline) nothing below it
    ImageDraw.Draw(im).rectangle([0, bot * S, W * S, H * S], fill=0)
    ids = Image.new("L", (W * S, H * S), 0)
    ImageDraw.Draw(ids).ellipse([qx0 * S, top * S, qx1 * S, bot * S], fill=2)
    ImageDraw.Draw(ids).polygon([(x * S, y * S) for x, y in pts], fill=2)
    ids.paste(Image.new("L", g.size, 1), (sx, top * S), g.point(lambda v: 255 if v > 127 else 0))
    idv = np.asarray(ids)
    seam = np.zeros(idv.shape, bool)
    for dx in range(-3 * S, 3 * S + 1, S):
        for dy in range(-3 * S, 3 * S + 1, S):
            seam |= (np.roll(np.roll(idv, dx, 1), dy, 0) != idv) & (idv > 0) & (np.roll(np.roll(idv, dx, 1), dy, 0) > 0)
    ring = Image.new("L", (W * S, H * S), 0)
    cx, cy, rr, wr = (qx0 + qx1) / 2 + h * 0.06, (top + bot) / 2 - h * 0.02, h * 0.25, max(2, int(h * 0.018 * S))
    ImageDraw.Draw(ring).ellipse([(cx - rr) * S, (cy - rr) * S, (cx + rr) * S, (cy + rr) * S], outline=255, width=wr)
    ring = seam.astype(float)              # (no ring engraved: the counter is open)
    return np.asarray(im, float) / 255, ring


def env(t):
    """The silver surround the chrome reflects (chrome, no colour, like THX's), by how far up (-1) or down (1)
    the reflected ray goes: dark grey high up to near white at the horizon, a sharp dark band, the ground dark to mid
    grey."""
    sky = np.interp(t, [-1, -0.05], [60, 245])
    ground = np.interp(t, [-0.05, 0.0, 0.15, 1], [20, 35, 120, 170])
    return np.where(t < -0.05, sky, ground)


def render3d(mask, ring, S=2):
    """The letters extruded and seen in perspective (a sharp 3D look, like HBO's): front faces flat and bright,
    the sides shaded by which way their edge faces, darker going back; the camera low and left, the letters receding
    to the right. Slices from the back to the front; each output pixel's ray met with each slice's plane."""
    mh, mw = mask.shape
    # the letters' plane: u across, v down, in frame pixels about the frame's middle; z back (depth D)
    D = 260.0
    # (the camera low and off to the left, an exaggerated foreshortening; on the ground, as the logo is: its
    # bottom a straight line) in close with a wide lens at the baseline's height, looking level toward the letters'
    # middle (no roll, no tilt: the uprights stay upright), the picture shifted so the baseline falls 80% down
    BASE = 176.0                     # the baseline (560 on the drawing, from its middle)
    cam = np.array([-560.0, BASE, -760.0])
    F = 900.0 * 1.25
    fwd = np.array([-cam[0], 0.0, -cam[2]]); fwd /= np.linalg.norm(fwd)
    right = np.cross(np.array([0.0, 1.0, 0.0]), fwd); right /= np.linalg.norm(right)
    down = np.cross(fwd, right)
    yy, xx = np.mgrid[0:H, 0:W].astype(float)
    dirs = np.stack([(xx - W / 2 - 76) / F, (yy - 0.80 * H) / F, np.ones_like(xx)], -1)     # (centred across)
    o = cam
    dv = dirs[..., 0:1] * right + dirs[..., 1:2] * down + dirs[..., 2:3] * fwd
    R = np.eye(3)
    # each edge's outward normal in the plane: from the blurred mask's slope
    blur = np.asarray(Image.fromarray((mask * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(3 * S)), float) / 255
    gy, gx = np.gradient(blur)
    nrm = np.stack([-gx, -gy], -1)
    nl = np.linalg.norm(nrm, axis=-1, keepdims=True) + 1e-9
    nrm = nrm / nl
    light = np.array([-0.5, -0.6, -0.62]); light /= np.linalg.norm(light)
    view = -dv / np.linalg.norm(dv, axis=-1, keepdims=True)
    # each pixel as the surround at an argument T (-1 up .. 1 down), times M, plus A (grey levels): what CARDIAC
    # works out from (segments/sqend)
    T = np.zeros((H, W)); M = np.zeros((H, W)); A = np.zeros((H, W))
    covered = np.zeros((H, W), bool)
    def sample(z):
        t = (z - o[2]) / dv[..., 2]
        pu = o[0] + t * dv[..., 0]
        pv = o[1] + t * dv[..., 1]
        mx = ((pu + W / 2) * S).astype(int); my = ((pv + H / 2) * S).astype(int)
        ok = (mx >= 0) & (mx < mw) & (my >= 0) & (my < mh)
        mxc, myc = np.clip(mx, 0, mw - 1), np.clip(my, 0, mh - 1)
        return ok, mxc, myc
    base = np.array([200.0, 198.0, 212.0])     # a cool light grey
    for z in np.linspace(D, 1.0, 110):
        ok, mxc, myc = sample(z)
        inside = ok & (mask[myc, mxc] > 0.5)
        n2 = nrm[myc, mxc]
        n3 = np.stack([n2[..., 0], n2[..., 1], np.zeros_like(n2[..., 0])], -1) @ R.T     # to the camera's frame
        diff = np.clip(-(n3 @ light), 0, 1)
        h = (-light + view); h /= np.linalg.norm(h, axis=-1, keepdims=True)
        spec = np.clip(np.sum(n3 * h, -1), 0, 1) ** 40
        shade = 0.32 + 0.6 * diff
        depth = 1 - 0.35 * (z / D)
        # the sides reflect the surround: the view mirrored in the side's normal, how far up or down it goes
        dotv = np.sum(view * n3, -1, keepdims=True)
        refl = view - 2 * dotv * n3
        T = np.where(inside, np.clip(-refl[..., 1] * 1.6, -1, 1), T)
        M = np.where(inside, 0.75 + 0.25 * depth, M)
        A = np.where(inside, 255 * spec * 0.5, A)
        covered |= inside
    # the front faces: flat, bright, a soft fall across; the ring engraved, dark
    ok, mxc, myc = sample(0.0)
    face = ok & (mask[myc, mxc] > 0.5)
    # the faces: the surround reflected straight on, by the row (the letters' top to bottom), the horizon 58% down
    rows = np.nonzero(face.any(1))[0]
    t = (yy - rows.min()) / max(1, rows.max() - rows.min()) * 2 - 1 + 0.16
    T = np.where(face, np.clip(t, -1, 1), T); M = np.where(face, 1.0, M); A = np.where(face, 0.0, A)
    # a thin bright line along the faces' edges (the bevel catching the light)
    edge = face & ~(np.roll(face, 2, 0) & np.roll(face, -2, 0) & np.roll(face, 2, 1) & np.roll(face, -2, 1))
    M = np.where(edge, 0.0, M); A = np.where(edge, 250.0, A)
    rg = face & (ring[myc, mxc] > 0.5)
    M = np.where(rg, 0.0, M); A = np.where(rg, 70.0, A)
    return T, M, A, covered | face


def sq_tma():
    """The SQ over the stars as (T, M, A) a pixel: the stars' pixels M 0, A their light."""
    mask, ring = letters_flat()
    T, M, A, cov = render3d(mask, ring)
    st = stars()[..., 0]
    return np.where(cov, T, 0.0), np.where(cov, M, 0.0), np.where(cov, A, st)


def stars(seed=7):
    rng = np.random.default_rng(seed)
    img = np.zeros((H, W, 3))
    for _ in range(140):
        x, y = rng.integers(0, W), rng.integers(0, H)
        b = rng.uniform(60, 230)
        img[y, x] = b
        if b > 190:
            for ddx, ddy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                if 0 <= x + ddx < W and 0 <= y + ddy < H: img[y + ddy, x + ddx] = b * 0.4
    return img


def name_layer():
    """'STRANGE QUERY' under the SQ (smaller, spaced out, chrome too): its cover, each pixel's chrome (the
    surround by the row over the letters' height, as the SQ's faces), and each pixel's turn to come in (0..1)."""
    im = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(im)
    f = ImageFont.truetype(FONT, 48)
    t = "STRANGE QUERY"
    track = 26                                  # the spacing between letters, pixels
    widths = [d.textlength(c, font=f) for c in t]
    total = sum(widths) + track * (len(t) - 1)
    x = (W - total) / 2
    for c, cw in zip(t, widths):
        d.text((x, 662), c, font=f, fill=255)
        x += cw + track
    cover = np.asarray(im, float) / 255
    rows = np.nonzero(cover.max(1) > 0.5)[0]
    yy = np.mgrid[0:H, 0:W][0].astype(float)
    tt = (yy - rows.min()) / max(1, rows.max() - rows.min()) * 2 - 1 + 0.16
    chrome_ = env(np.clip(tt, -1, 1))
    order = np.random.default_rng(1601).random((H, W))
    order = np.repeat(np.repeat(order[::3, ::3], 3, 0), 3, 1)[:H, :W]
    return cover, chrome_, order


_SQ = None


def sq_frame(f, ncover, nchrome, norder):
    """The SQ's frame f (from the hard cut): the letters in 3D over the stars; the name materialising from frame 8
    over 14 frames: each pixel, at its turn, a white sparkle for two frames, then its chrome."""
    global _SQ
    if _SQ is None:
        T, M, A = sq_tma()
        g = np.clip(env(T) * M + A, 0, 255)
        _SQ = np.stack([g] * 3, -1)
    img = _SQ.copy()
    u = (f - 8) / 14
    if u > 0:
        on = (norder < u) & (ncover > 0.3)
        spark = on & (norder > u - 2 / 14)
        col = np.where(spark[..., None], 255.0, np.stack([nchrome] * 3, -1))
        a = ncover[..., None] * on[..., None]
        img = img * (1 - a) + col * a
    return img


def cut_frames(first, n):
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", _work(*CUT_PREVIEW), "-vf",
                          f"select='between(n\\,{first}\\,{first + n - 1})',scale=1536:768", "-fps_mode", "passthrough",
                          "-f", "rawvideo", "-pix_fmt", "rgb24", "-"], capture_output=True).stdout
    return np.frombuffer(raw, np.uint8).reshape(-1, H, W, 3)


def preview(out):
    """The CARDIAC on the giant screen to the stadium's cut, then a hard cut to the SQ (no stutter at 12 fps)."""
    ncover, nchrome, norder = name_layer()
    before = cut_frames(STADIUM_AT + 455, BLACK_AT - 455)          # stadium 455..538
    p = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "1536x768",
                          "-r", "12", "-i", "-", "-vf", "scale=1024:512", "-c:v", "libx264", "-crf", "20", "-pix_fmt", "yuv420p",
                          "-movflags", "+faststart", out], stdin=subprocess.PIPE)
    for fr in before:
        p.stdin.write(fr.tobytes())
    still = None
    for f in range(36):
        img = sq_frame(f, ncover, nchrome, norder).astype(np.uint8)
        if f == 30: still = img
        p.stdin.write(img.tobytes())
    p.stdin.close(); p.wait()
    Image.fromarray(still).save(out.replace(".mp4", "_still.png"))


if __name__ == "__main__":
    if sys.argv[1] == "preview":
        preview(sys.argv[2])
