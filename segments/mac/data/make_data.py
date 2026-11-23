#!/usr/bin/env python3
"""The Mac cold open's data, made once from the prototype (prototypes/macdesk.py), whose drawing code it runs so the
pictures are its pixels exactly. Input data, like a font: nothing here is worked out per frame.

    python make_data.py       writes sprites.txt, tables.txt, titles.txt, dissolve.bin, boom.txt, haywire.txt and cues.txt beside it

- sprites.txt: every small picture the desk shows, as the prototype draws it: the icons (plain, selected, the floppy
  opened), the pointers (the arrow, the watch's eight hands), the Alarm Clock's small clock, the windows' parts of a
  fixed size (the close box and its starburst, the zoom box, the scroll arrows, the grow box, the Note Pad's turned
  corner), the alert's OK button (and pressed), the screen's rounded corners, the insertion point, the desktop's 50%
  grey over the whole screen (`desktop`). Each: `sprite NAME OX OY W H`, its place from its anchor, then H rows, `.`
  nothing, `o` white, `X` black.
- tables.txt: the tunnel's tables. `slope` for r = 0..127 (the slope r / 128 of a block's offset, smaller over larger,
  taken at the middle of its step): the angle in 800ths of a turn (0-100) and 200 (sec - 1), the secant less one in
  200ths (0-82). `ring` for each distance R2 = 0..620 from the middle in half pixels: how far along the tunnel (20 a
  check, 685.7 / R a check: 27429 / R2, at most 959), how far its depth twists it (1440 / R2, at most 60; the depth
  2400 / R by 0.6 in 50ths of a check), and its two shades darkened by the fog ((R - 12) / 170, from 0 to 1, to the
  power 1.3), light (0.92) and dark (0.42), 256 white. `sine`: 100 sin(0.075 i) for i = 0..105.
- titles.txt: the three title cards at the tunnel's 128x86 (Futura Medium, as the prototype sets them): `.` none,
  `X` the black rim, `o` the white letters.
- dissolve.bin: HyperCard's dissolve, the prototype's pixel order (numpy's generator seeded 1987): each pixel of the
  512x342 screen's turn, 255 r rounded down (0-254), a byte each, row by row. The prototype shows a pixel when r <
  j / 3: exactly when its turn is under 85 j.

The game (Boom) and the windows behind it (prototypes/macdoom.py, whose drawing code this runs too):

- sprites.txt also: the gun's four frames (`gun_KICK_FLASH`), the face (`face_LOOK_MOOD`), the status bar without its
  counters (`status_base`), the counters' digits (`big_0`..`big_9`, `big_pct`: white), the Wikipedia page's title and
  LEARN TO SPEAK / FINNISH in Chicago at twice the size (`wiki_title`, `finnish_title`).
- boom.txt: `map ROW` the map's 17 rows (0 empty; 1 brick, 2 steel panels, 3 stone, 4 the jambs, 5 the door);
  `tex KIND ROW` each texture's 64 rows of 64 greys (1-5 the walls, 6 the floor, 7 the ceiling; 256 white);
  `thresh E` for a light E (hundredths, 1-135) the 16 greys under which a texel is black at each step of the 4x4
  ordered dither, fl((2k + 1) 800, E) held to 999.
- cues.txt: the sound cues, frame by frame (`macboomcrash.py --marks`), for when the film plays sound.

The crash (prototypes/macboomcrash.py, with macpyramid.py's bomb box):

- sprites.txt also: the I-beam (`ibeam`); the system error's bomb at twice the size (`bomb`, its anchor the bomb's
  corner) and its buttons' rounded outline (`bomb_button`, its anchor the button's l, t); the editor's code, each line
  of bsp_tree.c in Monaco 9 at three times the size (`code_I`, its anchor the content's left edge on the line's
  baseline, the canvas's 4-pixel margin in it), as the prototype draws it before it triples the pixels.
- haywire.txt: the hall of mirrors' schedule, the prototype's numbers (its generator seeded 1993 and its formulas of
  the frame's place in the three seconds, worked out here once, like a typing rhythm): `band J X W` the 48 bands (each
  a place and width across the view); `tear Y DX` the four torn strips (22 rows from Y, moved DX across, wrapping);
  `frame F CLEAN BANDS SHIFT TEARS JUNK` for each frame from 32 to 67: whether it comes out clean (1), how many bands
  are spoiled, how far the smeared ones are moved across (wrapping), how many strips tear, and what the ammo counter
  shows (`-` itself).
"""
import math
import os
import sys

import numpy as np

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(here, "../../../prototypes"))
import macdesk as D  # noqa: E402
import macdoom as B  # noqa: E402
import macpyramid as Y  # noqa: E402
import macboomcrash as C  # noqa: E402

W, H = D.W, D.H


def capture(draw, x0, y0, w, h):
    """What draw(img) paints in the box (x0, y0, w, h): drawn on a white screen and on a black one, a pixel both left
    alone differs between them. Rows of '.', 'o', 'X'."""
    a, b = np.zeros((H, W), bool), np.ones((H, W), bool)
    draw(a)
    draw(b)
    a, b = a[y0:y0 + h, x0:x0 + w], b[y0:y0 + h, x0:x0 + w]
    return ["".join("." if a[j, i] != b[j, i] else ("X" if a[j, i] else "o") for i in range(w)) for j in range(h)]


def crop(rows, ox, oy):
    """Trim the rows to what is painted, the place from the anchor moved to match."""
    ys = [j for j, r in enumerate(rows) if r.strip(".")]
    xs = [i for i in range(len(rows[0])) if any(r[i] != "." for r in rows)]
    if not ys:
        return ox, oy, []
    rows = [r[xs[0]:xs[-1] + 1] for r in rows[ys[0]:ys[-1] + 1]]
    return ox + xs[0], oy + ys[0], rows


SPRITES = []


def sprite(name, draw, ax, ay, x0, y0, w, h):
    """A sprite drawn by draw(img) about the anchor (ax, ay), captured from the box (x0, y0, w, h)."""
    ox, oy, rows = crop(capture(draw, x0, y0, w, h), x0 - ax, y0 - ay)
    SPRITES.append((name, ox, oy, rows))


def icon(img, kind, x, y, selected):
    """The icon's own pixels, as draw_icon paints them (its label left out)."""
    ink, m = D.ICONS[kind]
    sub = img[y:y + 32, x:x + 32]
    if selected == "open":
        sub[m] = False
        yy, xx = np.mgrid[y:y + 32, x:x + 32]
        sub[m & ((xx + yy) % 2 == 0)] = True
        sub[D.edge_of(m)] = True
    elif selected:
        sub[m] = True
        sub[m & ink & np.pad(m, 1)[1:-1, 1:-1] & ~D.edge_of(m)] = False
        sub[D.edge_of(m)] = True
    else:
        sub[m] = False
        sub[ink] = True


def sprites():
    for kind in D.ICONS:
        for sel, tag in ((False, ""), (True, "_sel")):
            sprite(f"icon_{kind}{tag}", lambda img, k=kind, s=sel: icon(img, k, 100, 100, s), 100, 100, 100, 100, 32, 32)
    # the floppy opened, grey where its pixel's x + y is even: drawn where it sits on the desk
    sprite("icon_floppy_open", lambda img: icon(img, "floppy", 464, 76, "open"), 464, 76, 464, 76, 32, 32)
    sprite("arrow", lambda img: D.cursor(img, 100, 100, "arrow"), 100, 100, 80, 80, 40, 40)
    for k in range(8):
        sprite(f"watch{k}", lambda img, k=k: D.cursor(img, 100, 100, f"watch{k}"), 100, 100, 80, 80, 40, 40)
    small = np.array([[c == "X" for c in row] for row in D.SMALL_CLOCK])

    def clock(img):
        img[3:3 + small.shape[0], 100:100 + small.shape[1]] |= small
    sprite("menu_clock", clock, 100, 14, 90, 0, 30, 19)            # its anchor the menu's pen on the baseline

    def star(img):
        cx, cy = 100, 100
        for dx, dy in ((1, 0), (0, 1), (1, 1), (1, -1)):
            for k in range(-4, 5):
                if abs(k) > 1:
                    img[cy + dy * k, cx + dx * k] = True
    sprite("star", star, 100, 100, 90, 90, 20, 20)                 # its anchor the close box's middle
    sprite("close_box", lambda img: D.box(img, 108, 104), 100, 100, 100, 100, 30, 20)   # anchor: the window's l, t

    def zoom_box(img):
        D.box(img, 300 - 18, 104)
        D.frame(img, 300 - 18, 104, 300 - 13, 109)
    sprite("zoom_box", zoom_box, 300, 100, 270, 100, 31, 20)       # anchor: the window's r, t
    sprite("scroll_up", lambda img: D.arrow(img, 100, 100, up=True), 100, 100, 100, 100, 16, 17)
    sprite("scroll_down", lambda img: D.arrow(img, 100, 100, up=False), 100, 100, 100, 100, 16, 17)
    sprite("scroll_left", lambda img: D.harrow(img, 100, 100, left=True), 100, 100, 100, 100, 17, 16)
    sprite("scroll_right", lambda img: D.harrow(img, 100, 100, left=False), 100, 100, 100, 100, 17, 16)

    def grow(img):
        r, b = 300, 200
        D.frame(img, r - 11, b - 11, r - 3, b - 3)
        D.frame(img, r - 13, b - 13, r - 7, b - 7)
        D.fill(img, r - 12, b - 12, r - 8, b - 8, False)
    sprite("grow_box", grow, 300, 200, 280, 180, 21, 21)           # anchor: the window's r, b

    def fold(img):
        l, b = 100, 200
        D.fill(img, l, b - 14, l + 14, b - 14, True)
        for i in range(14):
            img[b - 14 + i, l + i] = True
    sprite("fold", fold, 100, 200, 100, 180, 20, 21)               # anchor: the Note Pad's page's l, b

    def ok(pressed):
        def draw(img):
            bl, bt, br, bb = D.OK_BUTTON
            for k in range(3):
                D.rounded(img, bl - 4 + k, bt - 4 + k, br + 4 - k, bb + 4 - k)
            D.rounded(img, bl, bt, br, bb, fill_with=pressed)
        return draw
    bl, bt, br, bb = D.OK_BUTTON
    sprite("ok", ok(False), bl, bt, bl - 6, bt - 6, br - bl + 13, bb - bt + 13)
    sprite("ok_pressed", ok(True), bl, bt, bl - 6, bt - 6, br - bl + 13, bb - bt + 13)
    for name, ax, ay, x0, y0 in (("corner_tl", 0, 0, 0, 0), ("corner_tr", W - 1, 0, W - 8, 0), ("corner_bl", 0, H - 1, 0, H - 8),
                                 ("corner_br", W - 1, H - 1, W - 8, H - 8)):
        sprite(name, D.screen_corners, ax, ay, x0, y0, 8, 8)
    # the insertion point: fill(x, base - 11, x, base + 2), its anchor the pen on the baseline
    sprite("caret", lambda img: D.fill(img, 100, 89, 100, 102, True), 100, 100, 95, 85, 10, 20)


def tables():
    out = ["# the Mac's tunnel tables: see make_data.py"]
    for r in range(128):
        s = (r + 0.5) / 128
        out.append(f"slope {r} {round(math.atan(s) * 800 / (2 * math.pi))} {round(200 * (math.sqrt(1 + s * s) - 1))}")
    for r2 in range(621):
        if r2 == 0:
            along, twist = 959, 60
        else:
            along, twist = min(959, round(27428.57 / r2)), min(60, round(1440 / r2))
        fog = min(1.0, max(0.0, (r2 / 2 - 12) / 170)) ** 1.3
        out.append(f"ring {r2} {along} {twist} {round(235.52 * fog)} {round(107.52 * fog)}")
    for i in range(106):
        out.append(f"sine {i} {round(100 * math.sin(0.075 * i))}")
    return out


def titles():
    from PIL import Image, ImageDraw, ImageFont
    out = ["# the title cards at 128x86: see make_data.py"]
    for k, (lines, size) in enumerate(([["STRANGE QUERY", "PRESENTS"], 9], [["A CARDIAC", "PRODUCTION"], 9], [["CARD", "ATTACK"], 22])):
        # title_card's own steps, kept at 128x86
        f = ImageFont.truetype(D.FUTURA, size, index=2)
        im = Image.new("L", (D.TW, D.TH), 0)
        d = ImageDraw.Draw(im)
        gap = int(size * 1.25)
        y = D.TH // 2 - gap * len(lines) // 2
        for line in lines:
            w = d.textlength(line, font=f)
            d.text(((D.TW - w) / 2, y), line, fill=255, font=f)
            y += gap
        white = np.array(im) > 127
        p = np.pad(white, 1)
        rim = white | p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:] | p[:-2, :-2] | p[2:, 2:] | p[:-2, 2:] | p[2:, :-2]
        big = lambda m: np.repeat(np.repeat(m, D.CHUNK, 0), D.CHUNK, 1)[:H, :W]
        assert (big(white) == D.TITLES[k][0]).all() and (big(rim) == D.TITLES[k][1]).all()
        out.append(f"title {k}")
        out += ["".join("o" if white[j, i] else "X" if rim[j, i] else "." for i in range(D.TW)) for j in range(D.TH)]
    return out


def dissolve():
    r = D.DISSOLVE
    turn = np.floor(r * 255).astype(np.uint8)
    for j in (1, 2, 3):
        assert ((r < j / 3) == (turn < 85 * j)).all()
    return turn.tobytes()


def bits(name, ink, mask, ax=0, ay=0):
    """A one-bit picture as a sprite: black where ink, white where the mask (and not ink), nothing elsewhere."""
    rows = ["".join("X" if ink[j, i] else ("o" if mask[j, i] else ".") for i in range(ink.shape[1])) for j in range(ink.shape[0])]
    SPRITES.append((name, -ax, -ay, rows))


def boom_sprites():
    for (kick, flash), (ink, mask) in B.GUNS.items():
        bits(f"gun_{kick}_{flash}", ink & mask, mask)
    for look in (-1, 0, 1):
        for mood in ("calm", "grin"):
            ink, m = B.face(look, mood)
            bits(f"face_{look}_{mood}", ink, m | ink)
    base = B.status_base()
    bits("status_base", base, np.ones_like(base))
    for c in "0123456789%":
        g = B.big_glyph(c)
        bits("big_pct" if c == "%" else f"big_{c}", np.zeros_like(g), g)
    bits("wiki_title", B.BIG_WIKI, B.BIG_WIKI)
    bits("finnish_title", B.BIG_FINNISH, B.BIG_FINNISH)


def crash_sprites():
    sprite("ibeam", lambda img: D.cursor(img, 100, 100, "ibeam"), 100, 100, 80, 80, 40, 40)

    def bomb(img):
        for y, row in enumerate(Y.BOMB_ICON):              # bomb_box's own steps, at (100, 100)
            for x, c in enumerate(row):
                if c == "X":
                    img[100 + 2 * y:102 + 2 * y, 100 + 2 * x:102 + 2 * x] = True
    sprite("bomb", bomb, 100, 100, 100, 100, 32, 32)
    sprite("bomb_button", lambda img: D.rounded(img, 100, 100, 170, 118), 100, 100, 100, 100, 71, 19)
    for i, line in enumerate(C.CODE.split("\n")):
        if not line:
            continue
        def draw(img, line=line):                          # editor_window's own steps, the line on the canvas's row 12
            c = np.zeros((30, 163), bool)
            D.text(c, 4, 12, line, Y.MONACO)
            big = c.repeat(C.ZOOM, 0).repeat(C.ZOOM, 1)
            img[100:100 + big.shape[0], 10:10 + big.shape[1]] |= big
        sprite(f"code_{i}", draw, 10, 100 + 3 * 12, 10, 100, 3 * 163, 90)


def haywire():
    out = ["# the hall of mirrors' schedule: see make_data.py"]
    for j, (x, w) in enumerate(C.BANDS):
        out.append(f"band {j} {x} {w}")
    for y0, dx in ((40, 37), (96, -52), (150, 61), (188, -29)):   # haywire()'s tears
        out.append(f"tear {y0} {dx}")
    for f in range(C.HAYWIRE, C.BOMB):                     # haywire()'s numbers, frame by frame
        u = (f - C.HAYWIRE) / (C.BOMB - C.HAYWIRE)
        clean = u < 0.4 and C.GLITCHY[f] > 0.25 + 1.5 * u
        n = 1 + int(len(C.BANDS) * u * u)
        shift = int(6 + 60 * u)
        tears = len(((40, 37), (96, -52), (150, 61), (188, -29))[:int((u - 0.5) * 9)]) if u >= 0.5 else 0
        junk = str([999, 0, 512, 666][f % 4]) if u >= 0.33 else "-"
        out.append(f"frame {f} {int(clean)} {n} {shift} {tears} {junk}")
    return out

def boom():
    out = ["# the game's data: see make_data.py"]
    for j, row in enumerate(B.MAP):
        out.append(f"map {j} " + " ".join(str(v) for v in row))
    texs = [B.TEX[1], B.TEX[2], B.TEX[3], B.TEX[4], B.TEX[5], B.FLOOR, B.CEIL]
    for k, t in enumerate(texs):
        for j in range(64):
            out.append(f"tex {k + 1} {j} " + " ".join(str(int(round(256 * v))) for v in t[j]))
    for e in range(1, 136):
        out.append(f"thresh {e} " + " ".join(str(min(999, (2 * k + 1) * 800 // e)) for k in range(16)))
    return out


def cues():
    frames, marks = C.script()
    out = ["# the sound cues: frame (12 a second), seconds, what (macboomcrash.py --marks)"]
    for f, w in marks:
        out.append(f"{f} {f / D.FPS:.2f} {w}")
    return out, len(frames)


def main():
    sprites()
    boom_sprites()
    crash_sprites()
    bits("desktop", D.desktop(), np.ones((H, W), bool))
    lines = ["# the Mac's sprites: see make_data.py"]
    for name, ox, oy, rows in SPRITES:
        lines.append(f"sprite {name} {ox} {oy} {len(rows[0]) if rows else 0} {len(rows)}")
        lines += rows
    open(os.path.join(here, "sprites.txt"), "w").write("\n".join(lines) + "\n")
    open(os.path.join(here, "tables.txt"), "w").write("\n".join(tables()) + "\n")
    open(os.path.join(here, "titles.txt"), "w").write("\n".join(titles()) + "\n")
    open(os.path.join(here, "dissolve.bin"), "wb").write(dissolve())
    open(os.path.join(here, "boom.txt"), "w").write("\n".join(boom()) + "\n")
    open(os.path.join(here, "haywire.txt"), "w").write("\n".join(haywire()) + "\n")
    c, n = cues()
    assert n == 377
    open(os.path.join(here, "cues.txt"), "w").write("\n".join(c) + "\n")
    print(len(SPRITES), "sprites")


if __name__ == "__main__":
    main()
