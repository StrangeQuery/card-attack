# Big lines for the cut: Inter Display Black, cap height 12, each line centred on 96x48, letters
# filled 1 with a one-pixel rim 2 around them (a 3x3 grow), lines 16 rows apart.
# python3 make_word.py out.txt "LINE" ["LINE" ...]
import sys, numpy as np
from PIL import Image, ImageDraw, ImageFont
FONT = '/usr/share/fonts/opentype/inter/InterDisplay-Black.otf'
def line(text, cap=12, track=2):
    for size in range(8, 60):
        f = ImageFont.truetype(FONT, size)
        if f.getbbox('H')[3] - f.getbbox('H')[1] >= cap: break
    top = f.getbbox('H')[1]
    cols = []
    for ch in text:
        if ch == ' ': cols.append(np.zeros((cap, 2), bool)); continue
        im = Image.new('L', (size * 2, size * 2), 0); ImageDraw.Draw(im).text((4, -top), ch, font=f, fill=255)
        a = np.array(im)[:cap] > 127; xs = np.nonzero(a.any(0))[0]
        cols.append(a[:, xs[0]:xs[-1] + 1]); cols.append(np.zeros((cap, track), bool))
    return np.concatenate(cols[:-1], 1)
def word(lines, pitch=16):
    L = [line(t) for t in lines]
    canvas = np.zeros((48, 96), bool)
    y0 = (48 - (pitch * (len(L) - 1) + 12)) // 2
    for k, a in enumerate(L):
        x0 = (96 - a.shape[1]) // 2; assert x0 >= 1, (lines[k], a.shape)
        canvas[y0 + k * pitch:y0 + k * pitch + 12, x0:x0 + a.shape[1]] |= a
    p = np.pad(canvas, 1); grow = np.zeros_like(canvas)
    for dy in range(3):
        for dx in range(3): grow |= p[dy:dy + 48, dx:dx + 96]
    return np.where(canvas, 1, np.where(grow, 2, 0))
if __name__ == '__main__':
    out = word(sys.argv[2:])
    open(sys.argv[1], 'w').write('\n'.join(''.join(map(str, r)) for r in out) + '\n')
