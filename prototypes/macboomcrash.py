"""The Mac cold open, Boom crashing (combine the gags): Boom as macdoom.py plays it, down the
corridor, through the door, past the pillars; then it goes haywire (the hall of mirrors: bands of the view not redrawn, smearing as
the player turns; walls from behind the player drawn over the ones in front; the ammo counter junk), worse and worse;
then the bomb box (Sorry, a system error occurred. ID = 02), the broken picture frozen behind it; Resume, the game
dies, back in THINK C's editor with its source, bsp_tree.c, at three times size: the plea (/* why doesn't this
work??? */) and, scrolled into view under it with two clicks, the line with its sign the wrong way round (the side
test that puts the walls in the wrong order); then the reminder. A second with the crash before Resume, three with the code before the reminder:
118 frames before the reminder, 58 more than Boom's; from the reminder on it is macdoom.py's script.

    python macboomcrash.py OUT.mp4 [--small SMALL.mp4] [--marks]
"""
import sys

import numpy as np

import macdesk as D
import macdoom as M
import macpyramid as Y

NAME = M.NAME
CW, CH = M.VW, M.VH + M.SBH
BOMB = 68                                                 # Boom's five seconds and a little more, then the crash
HAYWIRE = BOMB - 36                                       # three seconds to see it coming, worse and worse
RESUME = BOMB + 14                                        # a second to take in the crash, then to Resume
REMIND = RESUME + 36                                      # three seconds with the code before the reminder
GAME_FRAMES = REMIND                                      # 118: 58 frames longer than Boom's 60
ALARM = REMIND - 12
SCROLLS = (REMIND - 12, REMIND - 8)                       # the down arrow clicked twice, a line each (scroll a bit)

CODE = """/* bsp_tree.c */
/* draws the walls */
#include "boom.h"

void DrawNode(Node *n)
{
  int side = Side(n);
  /* why doesn't this
     work??? */
  if (side < 0) {
    DrawNode(n->back);
    DrawWall(n->wall);
    DrawNode(n->front);
  } else {
    DrawNode(n->front);
    DrawWall(n->wall);
    DrawNode(n->back);
  }
}"""
EDIT_RECT = (2, 21, 509, 339)                             # the editor's window, nearly the whole screen
ZOOM = 3                                                  # the code at three times size, to read across the room
SCROLL = {"s": 0}                                         # the first line shown
# scrolled two lines, the plea and the line it's about sit below where the reminder comes up


def editor_window():
    def draw(img, l, t, r, b, active):
        """The code in Monaco 9 at three times size: drawn at its own size, then each pixel tripled."""
        h, w = (b - t + 1) // ZOOM, (r - l + 1) // ZOOM
        c = np.zeros((max(h, 16 + 11 * CODE.count("\n")) + 16, w), bool)   # (the lines past the bottom cut off)
        for i, line in enumerate(CODE.split("\n")[SCROLL["s"]:]):
            D.text(c, 4, 12 + 11 * i, line, Y.MONACO)
        big = c[:h].repeat(ZOOM, 0).repeat(ZOOM, 1)
        img[t:t + big.shape[0], l:l + big.shape[1]] = big
    return {"title": "bsp_tree.c", "rect": EDIT_RECT, "scroll": True, "zoom": True, "draw": draw}


RNG = np.random.default_rng(1993)
BANDS = [(int(x), int(w)) for x, w in zip(RNG.integers(0, CW - 40, 48), RNG.integers(12, 70, 48))]
GLITCHY = RNG.random(BOMB)                                # early on, which frames show it at all


def haywire(f, last):
    """The hall of mirrors at game frame f, given the picture before, growing over the three seconds before the crash
    (u 0 to 1): at first a band or two now and then, the frame after clean again; then more and more bands left as
    they were (smeared sideways as the player turns), streaked (one column across the band), or showing the walls
    behind; the ammo counter junk from a third of the way; the rows tearing in the last half."""
    u = (f - HAYWIRE) / (BOMB - HAYWIRE)
    fr = M.game_frame(f)
    if u < 0.4 and GLITCHY[f] > 0.25 + 1.5 * u:          # early: most frames still come out clean
        return fr
    px, py, a, _, _ = M.player(f)
    behind = M.view(px, py, a + np.pi, M.door_open(f))
    out = fr.copy()
    n = 1 + int(len(BANDS) * u * u)
    for j, (x, w) in enumerate(BANDS[:n]):
        sl = slice(x, min(CW, x + w))
        if j % 3 == 2:
            out[:M.VH, sl] = behind[:, sl]                # walls behind drawn over the ones in front
        elif j % 3 == 1:
            out[:M.VH, sl] = out[:M.VH, x:x + 1]          # one column streaked across the band
        else:
            out[:M.VH, sl] = np.roll(last[:M.VH], int(6 + 60 * u), axis=1)[:, sl]   # not redrawn: smeared
    if u >= 0.5:                                          # and the rows tear
        for y0, dx in ((40, 37), (96, -52), (150, 61), (188, -29))[:int((u - 0.5) * 9)]:
            out[y0:y0 + 22, :] = np.roll(out[y0:y0 + 22, :], dx, axis=1)
    if u >= 0.33:
        out[M.VH:] = M.status_bar([999, 0, 512, 666][f % 4], 100, 50, 0, "calm")
    return out


def script():
    s = D.Script()
    gw = {"title": NAME, "rect": M.GAME_RECT, "scroll": False, "zoom": False}
    pic = {"frame": None, "bomb": False}

    def game_draw(img, l, t, r, b, active):
        img[t:t + CH, l:l + CW] = pic["frame"]
        if pic["bomb"]:
            Y.bomb_box(img)
    gw["draw"] = game_draw
    s.S["windows"] = [M.gif_window(), M.poem_window(), gw]
    s.S["menus"] = list(M.MENUS["game"])
    s.S["cursor"], s.S["mouse"] = None, (300.0, 200.0)
    s.mark("the game: the soundtrack starts")
    held = None
    for f in range(GAME_FRAMES):
        if f in M.SHOTS and f < BOMB:
            s.mark("gunshot")
        if f == M.DOOR_AT:
            s.mark("the door rises: door-open sound (about 0.8 s)")
        if f == 0:
            s.mark("footsteps while running (to the door, and again through it at frame 22)")
        if f < HAYWIRE:
            held = M.game_frame(f)
        elif f < BOMB:
            if f == HAYWIRE:
                s.mark("haywire starts: a glitch now and then, worse and worse over 3 s; the music starts to stutter")
            held = haywire(f, held)
        if f == BOMB:
            s.mark("the system error: the bomb box (the music stops dead)")
            pic["bomb"] = True
            s.S["cursor"], s.S["mouse"] = "arrow", (320.0, 150.0)
        if BOMB + 8 <= f <= BOMB + 12:                    # after a second, to Resume
            u = (f - BOMB - 8) / 4
            s.S["mouse"] = (320.0 + (Y.BOMB_RECT[2] - 46.0 - 320.0) * u, 150.0 + (Y.BOMB_RECT[3] - 20.0 - 150.0) * u)
        if f == RESUME:
            s.mark("Resume clicked: the game dies; back in THINK C's editor")
            pic["bomb"] = False
            s.S["windows"].pop()
            s.S["windows"].append(editor_window())
            s.S["menus"] = list(Y.EDITOR_MENUS)
            s.S["mouse"] = (440.0, 300.0)
            s.S["cursor"] = "ibeam"
        if SCROLLS[0] - 6 <= f <= SCROLLS[0] - 2:         # to the scroll bar's down arrow
            u = (f - SCROLLS[0] + 6) / 4
            ax, ay = EDIT_RECT[2] - 7.0, EDIT_RECT[3] - 24.0
            s.S["cursor"] = "arrow"
            s.S["mouse"] = (440.0 + (ax - 440.0) * u, 300.0 + (ay - 300.0) * u)
        if f in SCROLLS:
            if f == SCROLLS[0]:
                s.mark("the code scrolled: two clicks on the down arrow")
            SCROLL["s"] += 1
        if f == ALARM:
            s.mark("the Alarm Clock goes off: beeps with each flash (4)")
        if f >= ALARM:
            k = (f - ALARM) // 3
            s.S["menus"][0] = "CLOCK" if k % 2 == 0 else D.APPLE
        pic["frame"] = held
        s.snap()
    s.S["menus"][0] = D.APPLE
    s.S["cursor"], s.S["mouse"] = "arrow", (300.0, 230.0)
    s.mark("the reminder: alert chime")
    s.alert(D.REMINDER, read=16)
    M.close(s, "teach", read=6)                           # the editor, the Gouraud page, Learn to Speak Finnish
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
    M.script = lambda name=NAME: script()
    sys.argv = [sys.argv[0], args[0]] + (["--small", small] if small else [])
    M.main()


if __name__ == "__main__":
    main()
