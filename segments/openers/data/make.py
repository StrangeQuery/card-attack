# Makes the opening cards' data, once, from the cards' words. Nothing here draws a
# frame: the machines fade every card in and out.
# - strange.txt, couple.txt, ghz.txt, enjoy.txt: each card's lines in Futura Medium (the demo's lettering), centred
#   on the 192x96 screen, as bitmaps (1 a letter, 0 not), cut at 110 of 255.
# - photo.bin: the first card as a colour picture, 192x96, red, green, blue (0-255) a pixel, row by row: a
#   photo of the author's CARDIAC (the ending's cut-out, segments/ending/data/cardiac.bin, not in this repository,
#   laid on black and box-filtered to 88x84 at (6, 6)) on the left half, and THIS IS / A CARDIAC. / / IT RUNS AT / 0.2 HZ. in white, centred on the
#   right half, set as the bitmaps are.
# Python 3 with numpy and Pillow. Arg: the output folder (run from segments/openers/data).
import os
import sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont
OUT = sys.argv[1]
W, H = 192, 96
FUTURA = '/System/Library/Fonts/Supplemental/Futura.ttc'
PHOTO = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../ending/data/cardiac.bin')

def words(lines, size, box=(0, 0, W, H)):
    f = ImageFont.truetype(FUTURA, size, index=2)
    m = Image.new('L', (W, H)); d = ImageDraw.Draw(m)
    x0, y0, x1, y1 = box
    pitch = int(size * 1.35)                         # line to line
    top = y0 + ((y1 - y0) - pitch * len(lines)) / 2 + (pitch - size) / 2
    for i, text in enumerate(lines):
        b = d.textbbox((0, 0), text, font=f)
        d.text((x0 + ((x1 - x0) - (b[2] - b[0])) / 2 - b[0], top + i * pitch - b[1] + 1), text, font=f, fill=255)
    return np.asarray(m) > 110

for name, lines, size in [('strange.txt', ['STRANGE QUERY', 'WANTED TO SEE', 'WHAT IT COULD', 'REALLY DO'], 13),
                          ('couple.txt', ['SO WE GOT', 'A COUPLE OF THEM', 'AND OVERCLOCKED', 'THEM...'], 13),
                          ('ghz.txt', ['...TO 1 GHZ.'], 22),
                          ('enjoy.txt', ['ENJOY!'], 22)]:
    with open(f'{OUT}/{name}', 'w') as out:
        for r in words(lines, size): out.write(''.join(str(int(v)) for v in r) + '\n')

a = np.fromfile(PHOTO, np.uint8).reshape(648, 679, 4).astype(np.float32)
laid = np.clip(a[..., :3] * (a[..., 3:] / 10), 0, 255).astype(np.uint8)
card = np.zeros((H, W, 3), np.uint8)
card[6:90, 6:94] = np.asarray(Image.fromarray(laid).resize((88, 84), Image.BOX))
card[words(['THIS IS', 'A CARDIAC.', '', 'IT RUNS AT', '0.2 HZ.'], 12, (96, 0, 192, 96))] = 255
card.tofile(f'{OUT}/photo.bin')
