"""Degauss over the Mode X ray tracer: a colour blotch creeps in, then the degauss: the picture jolts and wobbles,
the colours smear and swirl, decaying over a second, and it comes back clean. A monitor's DEGAUSS sign (a warning
sign and the word, the VGA's 8x16 face at 6 x 6, on a dark box) comes up half a second before the jolt, blinking,
then stays up through it, shaking with the picture (it is laid on before the distortion), gone as the picture settles.

    python degauss.py MODEX_VIDEO_HD OUT.mp4 [FIRST] [raw]

MODEX_VIDEO_HD is segments/modex/video_hd.rgb (156 frames at 1536x768); frames FIRST.. (default 102) are shown, or
with `raw` written as raw RGB for comparing.

The design first ran over frames 0..95 of a 120-frame render, the blotch from frame 24 and the jolt at 48. Mode X
now has 36 frames more and the beat sits in them: the blotch from 114, creeping 20 frames rather than 24; the jolt at
134, its wobble's phase the same as the design's; and the sign."""
import subprocess, sys
from pathlib import Path
import numpy as np
W, H = 1536, 768
v = np.memmap(sys.argv[1], np.uint8, "r").reshape(-1, H, W, 3)
out = sys.argv[2]
first = int(sys.argv[3]) if len(sys.argv) > 3 else 102
raw = len(sys.argv) > 4 and sys.argv[4] == "raw"
if raw:
    sink = open(out, "wb")
else:
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", "12",
        "-i", "-", "-vf", "scale=1024:512", "-c:v", "libx264", "-profile:v", "main", "-level", "4.0", "-crf", "20", "-bf", "0",
        "-g", "12", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out], stdin=subprocess.PIPE)
    sink = ff.stdin
yy, xx = np.mgrid[0:H, 0:W].astype(np.float32)
B0, D0, DN = 114, 134, 16                                  # the blotch creeps from 114, degauss at 134 for 16 frames
PHASE0 = 48                                                # the wobble's phase as the first design's (its jolt at 48)
SIGN_ON, SIGN_OFF = D0 - 6, D0 + 12                        # the sign: blinking 3 on, 3 off from 128, solid from 134
def blotch(img, s):
    """Colour impurity: a magnetised patch, lower left and upper right, hue pushed toward purple and green."""
    d1 = np.exp(-(((xx - 330) / 420) ** 2 + ((yy - 560) / 300) ** 2))
    d2 = np.exp(-(((xx - 1250) / 360) ** 2 + ((yy - 170) / 260) ** 2))
    f = img.astype(np.float32)
    lum = f.mean(2) / 255                                  # a blotch only shows where the screen is lit
    d1 = d1 * lum; d2 = d2 * lum
    f[..., 0] += s * (70 * d1 - 40 * d2); f[..., 1] += s * (-50 * d1 + 60 * d2); f[..., 2] += s * (80 * d1 - 30 * d2)
    return f

# the sign: the box, the warning sign, the word (bold: drawn twice, a pixel apart), each pixel 6 x 6
seg = Path(__file__).resolve().parent.parent / "segments"
atlas = (seg / "ui/data/atlas.bin").read_bytes()
glyph = {}
for l in (seg / "ui/data/atlas.txt").read_text().splitlines():
    p = l.split()
    if p and p[0] == "g" and p[1] == "11": glyph[int(p[2])] = (int(p[8]), int(p[6]), int(p[7]))
word = np.zeros((16, 57), bool)
for dx in (0, 1):
    for n, ch in enumerate("DEGAUSS"):
        at, gw, gh = glyph[ord(ch)]
        cells = np.frombuffer(atlas[at:at + gw * gh], np.uint8).reshape(gh, gw) > 0
        word[:, dx + 8 * n:dx + 8 * n + 8] |= cells
sign = [l for l in (seg / "modex/data/warning.txt").read_text().splitlines() if not l.startswith("#")]
BOX = (489, 72, 1047, 228)                                 # x0, y0, x1, y1
ICON = (519, 96)
WORD = (675, 102)
GREEN, YELLOW = (85, 255, 85), (255, 255, 85)
def lay_sign(f):
    x0, y0, x1, y1 = BOX
    f[y0:y1, x0:x1] *= 0.25                                # the dark box
    big = np.kron(word, np.ones((6, 6), bool))
    f[WORD[1]:WORD[1] + 96, WORD[0]:WORD[0] + 342][big] = GREEN
    for r, line in enumerate(sign):
        for c, ch in enumerate(line):
            if ch != ".": f[ICON[1] + 6 * r:ICON[1] + 6 * r + 6, ICON[0] + 6 * c:ICON[0] + 6 * c + 6] = YELLOW if ch == "Y" else (0, 0, 0)
    return f
def sign_shown(k):
    return SIGN_ON <= k < D0 and (k - SIGN_ON) % 6 < 3 or D0 <= k < SIGN_OFF

for k in range(first, len(v)):
    f = v[k].astype(np.float32)
    if B0 <= k < D0:
        f = blotch(v[k], (k - B0) / (D0 - B0))
        if sign_shown(k): f = lay_sign(f)
    elif D0 <= k < D0 + DN:
        t = (k - D0) / DN
        a = 46 * np.exp(-4.0 * t)                           # the jolt, decaying
        base = blotch(v[k], max(0.0, 1 - 2.5 * t))
        if sign_shown(k): base = lay_sign(base)
        ph = (k - D0 + PHASE0) * 2.1
        dx = a * np.sin(yy / 37 + ph) + 0.5 * a * np.sin(yy / 11 - ph * 1.7)
        dy = 0.35 * a * np.sin(xx / 90 + ph * 1.3)
        o = np.empty_like(base)
        for c, (sx, sy) in enumerate(((1.0, 0.3), (0.0, 0.0), (-1.0, -0.3))):   # the guns land apart: colour fringes
            X = np.clip(xx + dx + sx * a * 0.5, 0, W - 1).astype(np.int32)
            Y = np.clip(yy + dy + sy * a * 0.5, 0, H - 1).astype(np.int32)
            o[..., c] = base[Y, X, c]
        sw = np.exp(-4.0 * t) * 60 * (base.mean(2) / 255)                          # a rainbow swirl washing round the screen
        ang = np.arctan2(yy - H / 2, xx - W / 2) + ph
        o[..., 0] += sw * np.sin(ang); o[..., 1] += sw * np.sin(ang + 2.1); o[..., 2] += sw * np.sin(ang + 4.2)
        o *= 1 + 0.25 * np.exp(-8 * t)                      # the flash as it kicks in
        f = o
    sink.write(np.clip(f, 0, 255).astype(np.uint8).tobytes())
sink.close()
if not raw: ff.wait()
