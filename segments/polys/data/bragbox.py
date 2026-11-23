# Dope's brag box, redrawn: a dark charcoal box with a soft bevel; the number in a wide face, PHONG in tall condensed
# capitals filling the width, polys in a round lowercase face, spaced out; chrome letters (white at the top, grey at
# the bottom) with a dark drop shadow, or gold, or bright. Drawn at 4x, then scaled down. Writes box.npy (RGB,
# 52x37). Usage: bragbox.py NUMBER [gold|bright]; boxes.sh draws them all.
import sys, numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter
S = 4; BW, BH = 52 * S, 37 * S
FUT = '/System/Library/Fonts/Supplemental/Futura.ttc'
def word(text, path, index, height, width=None, track=0):
    # the word's mask, `height` tall; stretched (or squeezed) to `width` if given
    f = ImageFont.truetype(path, 200, index=index)
    chars = [(c, f.getbbox(c)) for c in text]
    total = sum(b[2] - b[0] for _, b in chars) + track * 200 * (len(text) - 1)
    top = min(b[1] for _, b in chars); bot = max(b[3] for _, b in chars)
    im = Image.new('L', (int(total) + 20, bot - top + 20), 0); d = ImageDraw.Draw(im); x = 10
    for c, b in chars: d.text((x - b[0], 10 - top), c, font=f, fill=255); x += b[2] - b[0] + track * 200
    im = im.crop(im.getbbox())
    w = width or int(im.width * height / im.height)
    return im.resize((w, height), Image.LANCZOS)
style = sys.argv[2] if len(sys.argv) > 2 else 'chrome'
lines = [word(sys.argv[1], FUT, 4, 7 * S, None, 0.12),               # the number: Futura bold, spread wide
         word("PHONG", '/System/Library/Fonts/Supplemental/Impact.ttf', 0, 15 * S, 44 * S),
         word("polys", FUT, 2, 8 * S, None, 0.16)]                      # Futura medium, spaced
box = Image.new('RGB', (BW, BH))
g = np.linspace(0, 1, BW)[None, :] * 0.6 + np.linspace(0, 1, BH)[:, None] * 0.4
arr = (70 - 34 * g)[..., None].repeat(3, 2); arr[:S] += 22; arr[:, :S] += 14; arr[-S:] -= 16; arr[:, -S:] -= 12   # the bevel
box = Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8))
y = 2 * S
for i, m in enumerate(lines):
    x = (BW - m.width) // 2 if i == 1 else 4 * S
    shadow = Image.new('L', box.size, 0); shadow.paste(m, (x + S, y + S))
    shadow = shadow.filter(ImageFilter.GaussianBlur(S * 0.6))
    a = np.array(box).astype(float); sh = np.array(shadow)[..., None] / 255
    a = a * (1 - 0.75 * sh)
    # chrome: white at the top of the line, grey at the bottom, a darker band just below the middle
    h = m.height; ramp = np.linspace(0, 1, h)
    tone = 250 - 110 * ramp - 40 * np.exp(-((ramp - 0.58) / 0.08) ** 2)
    if style == 'bright': tone = 255 - 55 * ramp - 30 * np.exp(-((ramp - 0.58) / 0.08) ** 2)
    mask = np.zeros(box.size[::-1]); mask[y:y + h, x:x + m.width] = np.array(m) / 255
    # gold from the 7801 on; brighter, nearly white-hot, for the final count
    tint = {'gold': np.array([1.0, 0.78, 0.3]), 'bright': np.array([1.0, 0.93, 0.62])}.get(style, np.array([1.0, 1.0, 1.02]))
    col = np.zeros_like(a); col[y:y + h, x:x + m.width] = tone[:, None, None] * tint
    a = a * (1 - mask[..., None]) + col * mask[..., None]
    box = Image.fromarray(np.clip(a, 0, 255).astype(np.uint8))
    y += h + (1 * S if i == 0 else 2 * S)
box = box.resize((52, 37), Image.LANCZOS)
box.save('box.png'); np.save('box.npy', np.array(box))
box.resize((52 * 8, 37 * 8), Image.NEAREST).save('box_big.png')
