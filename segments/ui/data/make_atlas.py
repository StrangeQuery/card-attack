#!/usr/bin/env python3
"""The UI kit's coverage atlas: input data, made once here, like the ending's photo. Each glyph of the faces the UI
uses, rasterised from the system's Helvetica by PIL, and the kit's sprites (the cursor, the spinner's spokes, the
icons), drawn four times over and averaged. Every pixel is a coverage, 0-32 (32nds). The machines never see a font:
they place each glyph by its advance and offset, and the host fetches its coverages by index.

    python make_atlas.py       writes atlas.txt (the index) and atlas.bin (the coverages, a byte each) beside it

atlas.txt: `face ID SIZE BOLD MID` (MID: how far a middle-anchored line's top sits above its anchor), then
`g FACE CODE ADVANCE OX OY W H OFFSET` per glyph: its advance, its bitmap's place from the pen (at the line's top),
its size and where its coverages start in atlas.bin. Face 9 is the sprites (CODE: the sprite's number, its place
from the sprite's anchor).

Faces 10 and on are the PC's bitmap fonts, for text pages (pages.rs): one bit, every pixel 0 or 32, each glyph its
whole cell (8 wide, the face's height tall) from the pen at the cell's top left, rasterised by PIL from VileR's
Px437 fonts (the Ultimate Oldschool PC Font Pack; CREDITS.md) exactly as the prototypes drew them (bbs.py, vga.py).
Code 1 of a bitmap face is the text cursor: its cell's bottom two rows.

Faces 12 and on are for the terminal and the stream chat ("make me a demo", segments/claudedive): Menlo 30, regular
and bold, with the spinner's stars, and code 1 the terminal's block cursor (its place from the pen, no advance);
Arial 22, regular and bold, with the colour emoji the chat's messages carry (26 pixels, an advance of 28, as chat.py
pastes them); face 16 the colour emoji that float up beside the chat (56 pixels, their top left the anchor). The emoji
are Apple's (Apple Color Emoji), rasterised exactly as the prototype draws them (chat.py's emoji()). A colour glyph's
coverages are its alpha; `rgb FACE CODE OFFSET` gives where its colours (red, green, blue a pixel, its bitmap's size)
start in atlas.bin.

Faces 17, 18 and 19 (after all the others) are the Mac's bitmap fonts, Chicago 12, Geneva 9 and Geneva 12 (Susan Kare's, from System 6;
joshua stein's BDFs in prototypes/macfonts, CREDITS.md): one bit, every pixel 0 or 32, each glyph its BDF bitmap,
its code the MacRoman byte (20 is Chicago's Apple), its place from the pen on the baseline (OY the bitmap's top, so
above the baseline is negative), exactly as the prototype drew them (macdesk.py)."""
import math
import os

from PIL import Image, ImageDraw, ImageFont

FONT = "/System/Library/Fonts/Helvetica.ttc"
here = os.path.dirname(os.path.abspath(__file__))
# (size, bold): the faces the scenes use; their numbers are scenes.rs's
FACES = [(22, 0), (22, 1), (24, 0), (24, 1), (25, 0), (26, 0), (26, 1), (28, 1), (30, 1)]
CHARS = [chr(c) for c in range(32, 127)] + ["“", "”", "•"]
SPRITES = 9
SS = 4                                               # sprites drawn 4 x 4 times over
PCFONTS = os.path.join(here, "../../../prototypes/pcfonts")
# (file, the cell's height): the bitmap faces, from face 10, their numbers pages.rs's
BITMAP_FACES = [("Px437_IBM_CGA.ttf", 8), ("Px437_IBM_VGA_8x16.ttf", 16)]
CURSOR_CODE = 1
MENLO = "/System/Library/Fonts/Menlo.ttc"
ARIAL = "/System/Library/Fonts/Supplemental/Arial.ttf"
ARIAL_BOLD = "/System/Library/Fonts/Supplemental/Arial Bold.ttf"
ARIAL_BLACK = "/System/Library/Fonts/Supplemental/Arial Black.ttf"
# glide.py's pointer: X black, . white, each 3 x 3 pixels
ARROW = ["X", "XX", "X.X", "X..X", "X...X", "X....X", "X.....X", "X......X", "X.......X", "X........X", "X.....XXXXX",
         "X..X..X", "X.X X..X", "XX  X..X", "X    X..X", "     X..X", "      XX"]
EMOJI_FONT = "/System/Library/Fonts/Apple Color Emoji.ttc"
# the terminal's extra characters (claudedive.py): the ellipsis, the middle dot, the spinner's stars
MENLO_CHARS = CHARS + ["…", "·", "✢", "✳", "✶", "✻", "✽"]
# the emoji in the chat's messages, and the ones that float up (chat.py's CHAT and EMOJIS)
MESSAGE_EMOJI = "💀😭🤯😍"
FLOAT_EMOJI = "😂💀🥱👎🔥🤯😍👏🐟"
# the bitmap faces' glyphs past ASCII
EXTRA = "³║═╔╗╚╝"

MACFONTS = os.path.join(here, "../../../prototypes/macfonts")
# the Mac's bitmap faces, from face 17, after all the others (desk.rs's numbers)
MAC_FACES = ["Chicago-12", "Geneva-9", "Geneva-12"]


def cover(v):
    return (v * 32 + 127) // 255


def glyph(f, ch):
    """A glyph's coverages, cropped, and its place from the pen at the line's top (PIL's 'la' anchor)."""
    s = f.size
    im = Image.new("L", (4 * s, 4 * s), 0)
    ImageDraw.Draw(im).text((s, s), ch, font=f, fill=255, anchor="la")
    box = im.getbbox()
    if box is None:
        return 0, 0, 0, 0, b""
    x0, y0, x1, y1 = box
    data = bytes(cover(v) for v in im.crop(box).tobytes())
    return x0 - s, y0 - s, x1 - x0, y1 - y0, data


def sprite(draw, w, h, ox, oy):
    """A sprite drawn by draw(d, k) (k: the scale, 4) on a 4x canvas of w x h pixels; its anchor at (ox, oy) of the
    canvas. Returns its place from the anchor, its size and coverages (each pixel the mean of its 16)."""
    im = Image.new("L", (w * SS, h * SS), 0)
    draw(ImageDraw.Draw(im), SS)
    small = im.resize((w, h), Image.BOX)
    box = small.getbbox()
    x0, y0, x1, y1 = box
    data = bytes(cover(v) for v in small.crop(box).tobytes())
    return x0 - ox, y0 - oy, x1 - x0, y1 - y0, data


CURSOR = [(0, 0), (0, 44), (11, 34), (19, 52), (27, 48), (19, 31), (34, 31)]


def cursor(scale, part):
    """The arrow (its tip the anchor), scale 1 or 0.92 (pressed): part 0 its white (the outline and all), 1 its black
    (inside the outline)."""
    m = 4
    def draw(d, k):
        pts = [((m + u * scale) * k, (m + v * scale) * k) for u, v in CURSOR]
        d.polygon(pts, fill=255)
        if part == 1:
            d.line(pts + [pts[0]], fill=0, width=3 * k, joint="curve")
        else:
            d.line(pts + [pts[0]], fill=255, width=3 * k, joint="curve")
    return sprite(draw, 44, 62, m, m)


def spoke(k12):
    """The spinner's spoke k12 of 12 (its anchor the spinner's middle): 22 to 46 out, 9 wide."""
    c = 52
    a = 2 * math.pi * k12 / 12
    def draw(d, k):
        d.line([((c + 22 * math.cos(a)) * k, (c + 22 * math.sin(a)) * k), ((c + 46 * math.cos(a)) * k, (c + 46 * math.sin(a)) * k)], fill=255, width=9 * k)
    return sprite(draw, 2 * c, 2 * c, c, c)


def face():
    """The stand-in app icon's white face (its anchor the icon's top left, 76 pixels): ears and a face, eyes cut out."""
    s = 76 / 64
    def draw(d, k):
        q = s * k
        d.rounded_rectangle([12 * q, 20 * q, 52 * q, 46 * q], radius=11 * q, fill=255)
        d.polygon([(18 * q, 22 * q), (24 * q, 14 * q), (28 * q, 22 * q)], fill=255)
        d.polygon([(36 * q, 22 * q), (40 * q, 14 * q), (46 * q, 22 * q)], fill=255)
        for ex in (24, 40):
            d.ellipse([(ex - 4) * q, 29 * q, (ex + 4) * q, 38 * q], fill=0)
    return sprite(draw, 76, 76, 0, 0)


def quit_icon():
    """The Force Quit button's icon: a ring with a cross (its anchor the button's top left)."""
    def draw(d, k):
        d.ellipse([14 * k, 11 * k, 36 * k, 33 * k], outline=255, width=3 * k)
        d.line([20 * k, 17 * k, 30 * k, 27 * k], fill=255, width=3 * k)
        d.line([30 * k, 17 * k, 20 * k, 27 * k], fill=255, width=3 * k)
    return sprite(draw, 48, 44, 0, 0)


def bitmap_glyph(f, h, ch):
    """A bitmap face's glyph: its whole 8 x h cell, drawn at the cell's top left, each pixel 0 or 32 (PIL's pixels
    over half lit)."""
    im = Image.new("L", (8, h), 0)
    ImageDraw.Draw(im).text((0, 0), ch, fill=255, font=f)
    return bytes(32 if v > 127 else 0 for v in im.tobytes())


def emoji(ch):
    """A colour emoji as chat.py draws it: at 160 on a 170 x 170 canvas, scaled to 56 x 56 (LANCZOS), as floats."""
    import numpy as np
    im = Image.new("RGBA", (170, 170))
    ImageDraw.Draw(im).text((0, 0), ch, font=ImageFont.truetype(EMOJI_FONT, 160), embedded_color=True)
    return np.array(im.resize((56, 56), Image.LANCZOS), np.float32)


def colour_glyph(rgba):
    """A colour glyph cropped to its alpha: its place in the bitmap, size, coverages (alpha in 32nds) and colours."""
    im = Image.fromarray(rgba)
    x0, y0, x1, y1 = im.getchannel("A").getbbox()
    crop = rgba[y0:y1, x0:x1]
    covers = bytes(cover(int(v)) for v in crop[..., 3].ravel())
    return x0, y0, x1 - x0, y1 - y0, covers, crop[..., :3].tobytes()

def bdf(name):
    """A BDF font's glyphs: (code, advance, x offset, bitmap top from the baseline, w, h, the bitmap's pixels 0 or 32)."""
    glyphs, cur = [], None
    for line in open(os.path.join(MACFONTS, name + ".bdf"), encoding="latin-1"):
        k = line.split()
        if not k:
            continue
        if k[0] == "ENCODING":
            cur = {"code": int(k[1]), "rows": []}
        elif k[0] == "DWIDTH":
            cur["adv"] = int(k[1])
        elif k[0] == "BBX":
            cur["bbx"] = tuple(map(int, k[1:5]))
        elif k[0] == "BITMAP":
            cur["bits"] = True
        elif k[0] == "ENDCHAR":
            w, h, xo, yo = cur["bbx"]
            data = bytes(32 if (int(r, 16) >> (len(r) * 4 - 1 - i)) & 1 else 0 for r in cur["rows"] for i in range(w))
            assert len(data) == w * h
            glyphs.append((cur["code"], cur["adv"], xo, -(yo + h), w, h, data))
            cur = None
        elif cur is not None and cur.get("bits"):
            cur["rows"].append(k[0])
    return glyphs


def main():
    lines, blob = ["# the UI kit's atlas: see make_atlas.py"], bytearray()
    for fid, (size, bold) in enumerate(FACES):
        f = ImageFont.truetype(FONT, size, index=bold)
        mid = f.getbbox("A", anchor="la")[1] - f.getbbox("A", anchor="lm")[1]
        lines.append(f"face {fid} {size} {bold} {mid}")
        for ch in CHARS:
            ox, oy, w, h, data = glyph(f, ch)
            adv = f.getlength(ch)
            assert adv == int(adv), (size, ch, adv)
            lines.append(f"g {fid} {ord(ch)} {int(adv)} {ox} {oy} {w} {h} {len(blob)}")
            blob += data
    lines.append(f"face {SPRITES} 0 0 0")
    sprites = [cursor(1, 0), cursor(1, 1), cursor(0.92, 0), cursor(0.92, 1), face(), quit_icon()] + [spoke(k) for k in range(12)]
    for code, (ox, oy, w, h, data) in enumerate(sprites):
        lines.append(f"g {SPRITES} {code} 0 {ox} {oy} {w} {h} {len(blob)}")
        blob += data
    for k, (file, h) in enumerate(BITMAP_FACES):
        fid = SPRITES + 1 + k
        f = ImageFont.truetype(os.path.join(PCFONTS, file), h)
        lines.append(f"face {fid} {h} 0 0")
        lines.append(f"g {fid} {CURSOR_CODE} 8 0 0 8 {h} {len(blob)}")
        blob += bytes(32 if row >= h - 2 else 0 for row in range(h) for _ in range(8))
        for ch in CHARS[:95]:
            lines.append(f"g {fid} {ord(ch)} 8 0 0 8 {h} {len(blob)}")
            blob += bitmap_glyph(f, h, ch)
    # the terminal's Menlo (faces 12, 13), the chat's Arial (14, 15), the floating emoji (16)
    import numpy as np
    fid = SPRITES + 1 + len(BITMAP_FACES)
    for file, index, size, chars in ((MENLO, 0, 30, MENLO_CHARS), (MENLO, 1, 30, MENLO_CHARS), (ARIAL, 0, 22, CHARS), (ARIAL_BOLD, 0, 22, CHARS)):
        f = ImageFont.truetype(file, size, index=index)
        mid = f.getbbox("A", anchor="la")[1] - f.getbbox("A", anchor="lm")[1]
        lines.append(f"face {fid} {size} {int(file == ARIAL_BOLD or index == 1)} {mid}")
        if file == MENLO:
            # the block cursor: claudedive.py's rectangle, 2 to 18 across from the pen, 0 to 36 down from the text's top
            lines.append(f"g {fid} {CURSOR_CODE} 0 2 0 17 37 {len(blob)}")
            blob += bytes([32] * 17 * 37)
        for ch in chars:
            ox, oy, w, h, data = glyph(f, ch)
            adv = f.getlength(ch)
            assert adv == int(adv), (file, size, ch, adv)
            lines.append(f"g {fid} {ord(ch)} {int(adv)} {ox} {oy} {w} {h} {len(blob)}")
            blob += data
        if file == ARIAL and index == 0:
            # the emoji in a message: 26 x 26, pasted a pixel above the line's top, 28 apart (chat.py)
            for ch in MESSAGE_EMOJI:
                small = np.array(Image.fromarray(emoji(ch).astype(np.uint8)).resize((26, 26), Image.LANCZOS))
                ox, oy, w, h, covers, rgb = colour_glyph(small)
                lines.append(f"g {fid} {ord(ch)} 28 {ox} {oy - 1} {w} {h} {len(blob)}")
                blob += covers
                lines.append(f"rgb {fid} {ord(ch)} {len(blob)}")
                blob += rgb
        fid += 1
    lines.append(f"face {fid} 0 0 0")
    for ch in FLOAT_EMOJI:
        ox, oy, w, h, covers, rgb = colour_glyph(emoji(ch).astype(np.uint8))
        lines.append(f"g {fid} {ord(ch)} 0 {ox} {oy} {w} {h} {len(blob)}")
        blob += covers
        lines.append(f"rgb {fid} {ord(ch)} {len(blob)}")
        blob += rgb

    for k, name in enumerate(MAC_FACES):
        fid = sum(1 for l in lines if l.startswith("face "))     # the next face number, after all the others
        lines.append(f"face {fid} {name.split('-')[1]} 0 0")
        for code, adv, ox, oy, w, h, data in bdf(name):
            lines.append(f"g {fid} {code} {adv} {ox} {oy} {w} {h} {len(blob)}")
            blob += data
    # past ASCII, appended so every glyph above keeps its place: the BBS's box and its bar's divider (bbs.py's "³",
    # drawn as the font draws that character)
    for k, (file, h) in enumerate(BITMAP_FACES):
        fid = SPRITES + 1 + k
        f = ImageFont.truetype(os.path.join(PCFONTS, file), h)
        for ch in EXTRA:
            lines.append(f"g {fid} {ord(ch)} 8 0 0 8 {h} {len(blob)}")
            blob += bitmap_glyph(f, h, ch)
    # the accelerated drive's VIDEO OPTIONS menu and its brag (accel.py), after all the others: Arial Black 52 (the
    # menu's title), Arial Bold 34 (its rows, its values, APPLY), Arial Black 40 (the brag); and its pointer, glide.py's
    # pixel arrow at 3 x 3 (face 9's codes 18, its white, and 19, its black; its top left the anchor)
    for file, size in ((ARIAL_BLACK, 52), (ARIAL_BOLD, 34), (ARIAL_BLACK, 40)):
        fid = sum(1 for l in lines if l.startswith("face "))
        f = ImageFont.truetype(file, size)
        mid = f.getbbox("A", anchor="la")[1] - f.getbbox("A", anchor="lm")[1]
        lines.append(f"face {fid} {size} 1 {mid}")
        for ch in CHARS:
            ox, oy, w, h, data = glyph(f, ch)
            adv = f.getlength(ch)
            assert adv == int(adv), (file, size, ch, adv)
            lines.append(f"g {fid} {ord(ch)} {int(adv)} {ox} {oy} {w} {h} {len(blob)}")
            blob += data
    for code, part in ((18, "."), (19, "X")):
        w, h = 3 * max(len(r) for r in ARROW), 3 * len(ARROW)
        data = bytes(32 if c // 3 < len(ARROW[y // 3]) and ARROW[y // 3][c // 3] == part else 0 for y in range(h) for c in range(w))
        lines.append(f"g {SPRITES} {code} 0 0 0 {w} {h} {len(blob)}")
        blob += data
    # the party chat's (segments/partychat): the middle dot in Arial 22 (face 14), after all the others
    f = ImageFont.truetype(ARIAL, 22)
    for ch in "·":
        ox, oy, w, h, data = glyph(f, ch)
        adv = f.getlength(ch)
        assert adv == int(adv), ("arial 22", ch, adv)
        lines.append(f"g 14 {ord(ch)} {int(adv)} {ox} {oy} {w} {h} {len(blob)}")
        blob += data
    open(os.path.join(here, "atlas.txt"), "w").write("\n".join(lines) + "\n")
    open(os.path.join(here, "atlas.bin"), "wb").write(bytes(blob))
    print(f"{len(lines)} lines, {len(blob)} bytes")


if __name__ == "__main__":
    main()
