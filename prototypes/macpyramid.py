"""The Mac cold open, the pyramid version (a tumbling polygon that crashes sets up the demo angle
better than the shooter): in Boom's window, someone's 3D demo, a flat-shaded pyramid tumbling in one bit at a Plus's
pace; then the obvious bug: one corner's fixed-point sum wraps, the apex snaps to the far side of the window and the
faces stretch into spikes across it, flickering; then the bomb box (Sorry, a system error occurred. ID = 02), the
picture frozen behind it; Resume, the demo dies, back in THINK C's editor with the code and its plea (/* why
doesn't this work??? */), the line under it (its sign the wrong way round) selected with a drag, and the Alarm Clock
goes off over it. From the reminder on it is macdoom.py's
script unchanged (the Gouraud page, Learn to Speak Finnish, the floppy, the launch): the same 60 frames before it.

    python macpyramid.py OUT.mp4 [--small SMALL.mp4] [--marks]
"""
import sys

import numpy as np

import macdesk as D
import macdoom as M

NAME = "democode1"                                        # a beginner's name for it
GAME_FRAMES = M.GAME_FRAMES                               # 60: the rest of the cold open keeps its timing
CW, CH = M.VW, M.VH + M.SBH                               # the window's content, 448 x 271
STEP = 2                                                  # a new picture every other frame: six a second
BUG = 16                                                  # the corner first wraps
BOMB = 30                                                 # the system error
RESUME = 38                                               # Resume clicked: back to the editor
ALARM = GAME_FRAMES - 12                                  # the Alarm Clock, over the crash (as in Boom)

# the pyramid: a square base and its apex, unit size
V = np.array([[-1, -0.7, -1], [1, -0.7, -1], [1, -0.7, 1], [-1, -0.7, 1], [0, 1.1, 0]], float)
FACES = [(0, 1, 4), (1, 2, 4), (2, 3, 4), (3, 0, 4), (3, 2, 1, 0)]
LIGHT = np.array([-0.4, 0.6, -0.7]) / np.linalg.norm([-0.4, 0.6, -0.7])
BAYER = np.tile(M.BAYER, (CH // 4 + 1, CW // 4))[:CH, :CW]


def turn(step):
    a, b = 0.37 * step, 0.5 + 0.35 * np.sin(0.21 * step)
    ca, sa, cb, sb = np.cos(a), np.sin(a), np.cos(b), np.sin(b)
    ry = np.array([[ca, 0, sa], [0, 1, 0], [-sa, 0, ca]])
    rx = np.array([[1, 0, 0], [0, cb, -sb], [0, sb, cb]])
    return rx @ ry


def corners(step, wrap):
    """The corners on the window: the turn, 4 units away, the window's focal length. With `wrap`, the apex's x and
    y pass a 16-bit Fixed's range and come back from the other side: it lands in the window's far top corner."""
    p = V @ turn(step).T
    p[:, 2] += 4.0
    f = 300.0
    xy = np.c_[CW / 2 + f * p[:, 0] / p[:, 2], CH / 2 - f * p[:, 1] / p[:, 2]]
    if wrap:                                              # it lands in the window's far corner
        xy[4, 0] = 14 if xy[4, 0] > CW / 2 else CW - 14
        xy[4, 1] = 8                                      # (the top: above where the bomb box comes)
    return p, xy


def polygon_mask(pts):
    """The pixels whose middles are inside a convex polygon (either winding)."""
    yy, xx = np.mgrid[0:CH, 0:CW] + 0.5
    inside_pos = np.ones((CH, CW), bool)
    inside_neg = np.ones((CH, CW), bool)
    n = len(pts)
    for k in range(n):
        (x0, y0), (x1, y1) = pts[k], pts[(k + 1) % n]
        c = (x1 - x0) * (yy - y0) - (y1 - y0) * (xx - x0)
        inside_pos &= c >= 0
        inside_neg &= c <= 0
    return inside_pos | inside_neg


def picture(step, wrap):
    """The demo's picture (True ink): black, the faces back to front (the painter's), each a dithered grey by its
    light, a white edge round each."""
    p, xy = corners(step, wrap)
    img = np.ones((CH, CW), bool)
    order = sorted(range(len(FACES)), key=lambda k: -np.mean([p[i, 2] for i in FACES[k]]))
    for k in order:
        f = FACES[k]
        n = np.cross(p[f[1]] - p[f[0]], p[f[2]] - p[f[0]])
        n /= np.linalg.norm(n)
        if wrap or n[2] < 0:                              # facing us (the bug draws them all)
            grey = 0.15 + 0.85 * max(0.0, -n @ LIGHT) if not wrap else 0.15 + 0.85 * abs(n @ LIGHT)
            m = polygon_mask([xy[i] for i in f])
            img[m] = grey <= BAYER[m]
    return img


BOMB_RECT = (68, 212, 444, 300)                           # the system error's box (below the reminder, which comes over it)
BOMB_ICON = ["...........XX...", "..........X..X..", ".........X......", "....XXXXX.......", "....XXXXX.......",
             "..XXXXXXXXX.....", ".XXXXXXXXXXX....", ".XXX.XXXXXXX....", "XXX.XXXXXXXXX...", "XXX.XXXXXXXXX...",
             "XXXXXXXXXXXXX...", "XXXXXXXXXXXXX...", ".XXXXXXXXXXX....", ".XXXXXXXXXXX....", "..XXXXXXXXX.....",
             "....XXXXX......."]


def bomb_box(img):
    """System 6's system error: the double-bordered box, the bomb, the words, Restart and Resume."""
    l, t, r, b = BOMB_RECT
    D.fill(img, l, t, r, b, True)
    D.fill(img, l + 1, t + 1, r - 1, b - 1, False)
    D.fill(img, l + 3, t + 3, r - 3, b - 3, True)
    D.fill(img, l + 5, t + 5, r - 5, b - 5, False)
    for y, row in enumerate(BOMB_ICON):                   # the bomb at twice size
        for x, c in enumerate(row):
            if c == "X":
                img[t + 18 + 2 * y:t + 20 + 2 * y, l + 18 + 2 * x:l + 20 + 2 * x] = True
    D.text(img, l + 66, t + 32, "Sorry, a system error occurred.", D.CHICAGO)
    D.text(img, l + 66, t + 50, "ID = 02", D.CHICAGO)
    for label, x0 in (("Restart", r - 170), ("Resume", r - 86)):
        bl, bt, br, bb = x0, b - 30, x0 + 70, b - 12
        D.rounded(img, bl, bt, br, bb)
        D.text(img, (bl + br) // 2 - D.width(label, D.CHICAGO) // 2, bb - 5, label, D.CHICAGO)


MONACO = D.load_bdf("Monaco-9")
# the program, as THINK C's editor shows it: a beginner's, the plea, and under it the line with the sign the
# wrong way round (the rotation's + should be -; and int is 16 bits, so x * 256 overflows too)
CODE = """/* democode1.c -- my first demo!! */

#include <math.h>

int px[5] = { -50,  50,  50, -50,   0 };
int py[5] = { -35, -35, -35, -35,  55 };
int pz[5] = { -50, -50,  50,  50,   0 };

void DrawPyramid(int t)
{
    int i, x, y, z, sx[5], sy[5];

    for (i = 0; i < 5; i++) {
        /* why doesn't this work??? */
        x = px[i] * cos(t) + pz[i] * sin(t);
        z = px[i] * sin(t) + pz[i] * cos(t) + 200;
        y = py[i];
        sx[i] = 224 + x * 256 / z;
        sy[i] = 135 - y * 256 / z;
    }
    FillTriangles(sx, sy);
}"""
BAD = "x = px[i] * cos(t) + pz[i] * sin(t);"
SEL = {"n": 0}                                            # how much of the bad line is selected
SEL0, SEL1 = 51, 58                                       # the drag along it, ending as the reminder pops
EDITOR_MENUS = [D.APPLE, "File", "Edit", "Search", "Project", "Source"]


def editor_window():
    def draw(img, l, t, r, b, active):
        for i, line in enumerate(CODE.split("\n")):
            D.text(img, l + 6, t + 12 + 11 * i, line, MONACO)
            if line.strip() == BAD and SEL["n"] > 0:      # the selection, as far as the drag has gone
                start = len(line) - len(line.lstrip())
                x0 = l + 6 + D.width(line[:start], MONACO) - 1
                x1 = l + 6 + D.width(line[:start + SEL["n"]], MONACO)
                img[t + 3 + 11 * i:t + 14 + 11 * i, x0:x1] ^= True
    return {"title": "democode1.c", "rect": M.GAME_RECT, "scroll": True, "zoom": True, "draw": draw}


def script():
    s = D.Script()
    gw = {"title": NAME, "rect": M.GAME_RECT, "scroll": False, "zoom": False}
    pic = {"frame": None, "bomb": False}

    def game_draw(img, l, t, r, b, active):
        img[t:t + CH, l:l + CW] = pic["frame"]
        if pic["bomb"]:
            bomb_box(img)
    gw["draw"] = game_draw
    s.S["windows"] = [M.gif_window(), M.poem_window(), gw]
    s.S["menus"] = list(M.MENUS["game"])[:3]               # a beginner's program: no menu of its own
    s.S["cursor"], s.S["mouse"] = None, (300.0, 200.0)
    s.mark("the demo: a pyramid tumbling (a hum, a tick each new picture)")
    held = None
    for f in range(GAME_FRAMES):
        step = min(f, BOMB) // STEP
        if f == BUG:
            s.mark("the bug: the apex wraps; the spikes flicker")
        # past the bug the wrapped picture comes more and more often, and the crash freezes on one
        wrap = BUG <= f and (f >= BOMB or (f - BUG) % max(1, 4 - (f - BUG) // 5) == 0)
        if f < BOMB:
            held = picture(step, wrap)
        if f == BOMB:
            held = picture(step, True)
            s.mark("the system error: the bomb box (the hum stops dead)")
            pic["bomb"] = True
            s.S["cursor"], s.S["mouse"] = "arrow", (320.0, 150.0)
        if f == BOMB + 3:                                 # the pointer to Resume
            bl = BOMB_RECT[2] - 86
            s.S["mouse"] = (bl + 40.0, BOMB_RECT[3] - 20.0)
        if f == RESUME:
            s.mark("Resume clicked: the demo dies; back in THINK C's editor")
            pic["bomb"] = False
            s.S["windows"].pop()
            s.S["windows"].append(editor_window())
            s.S["menus"] = list(EDITOR_MENUS)
            s.S["mouse"] = (420.0, 300.0)                 # off the code, over the editor's text: the I-beam
            s.S["cursor"] = "ibeam"
        if SEL0 <= f <= SEL1:                             # the drag across the line with the wrong sign
            if f == SEL0:
                s.mark("the line with the wrong sign selected: a drag")
            u = (f - SEL0) / (SEL1 - SEL0)
            SEL["n"] = round(len(BAD) * u)
            i = CODE.split("\n").index("        " + BAD)
            l, t = M.GAME_RECT[0] + 1, M.GAME_RECT[1] + 19
            s.S["mouse"] = (l + 6 + D.width("        " + BAD[:SEL["n"]], MONACO), t + 10 + 11 * i)
        if f == SEL1 + 2:
            s.S["mouse"] = (420.0, 300.0)
        if f == GAME_FRAMES - 1:
            s.S["cursor"] = "arrow"
        if f == ALARM:
            s.mark("the Alarm Clock goes off over the crash: beeps with each flash (4)")
        if f >= ALARM:
            k = (f - ALARM) // 3
            s.S["menus"][0] = "CLOCK" if k % 2 == 0 else D.APPLE
        pic["frame"] = held
        s.snap()
    s.S["menus"][0] = D.APPLE
    s.S["cursor"], s.S["mouse"] = "arrow", (300.0, 230.0)
    s.mark("the reminder: alert chime")
    s.alert(D.REMINDER, read=16)
    M.close(s, "teach", read=6)                           # the editor's window, then as before
    M.close(s, "teach", read=8)
    M.close(s, "finder", read=4)
    D.launch(s)
    return s.frames, s.marks


def main():
    args = sys.argv[1:]
    if "--marks" in args:
        frames, marks = script()
        for f, w in marks:
            print(f"{f:4d}  {f / M.FPS:5.2f} s  {w}")
        print(len(frames), "frames")
        return
    small = None
    if "--small" in args:
        i = args.index("--small"); small = args[i + 1]; del args[i:i + 2]
    M.script = lambda name=NAME: script()                 # macdoom's preview writer, with this script
    sys.argv = [sys.argv[0], args[0]] + (["--small", small] if small else [])
    M.main()


if __name__ == "__main__":
    main()
