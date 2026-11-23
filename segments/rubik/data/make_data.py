# Makes the Rubik's cube's data files (run once, plain Python): the printed tables the machines read off the deck,
# the cube's materials and light, the moves of the prototype's solve, and the solved cube.
#   python make_data.py
# sine.txt       999 sin(k degrees), k = 0..91 (the last only to read past 90)
# pow10.txt      999 x^10, x = 0.50, 0.51, ... 1.00 (the plastic's highlight)
# pow24.txt      999 x^24, the same x (the stickers' highlight)
# materials.txt  the plastic, then the six sticker colours by home face (L, R, D, U, B, F), red, green, blue each,
#                in 64ths of a step of the 3-digit colour code (576 is full)
# light.txt      the unit vector toward the light, then the half-way vector, 999 each part
# moves.txt      the prototype's scramble (seed 1982), then its solve: axis (0 x, 1 y, 2 z), side (-1, 1),
#                quarter turns clockwise looking at that face (1, -1, 2)
# solved.txt     the solved cube's 54 stickers: cubie place (x, y, z), outward normal (x, y, z), colour (the home
#                face, 2 axis + (side > 0))
# cubies.txt     the 27 cubies, x outermost: place (x, y, z); the distance key's start, 250 |p|^2 / 18 rounded (the
#                eye's distance squared is 81 + |p|^2 - 18 depth, in units); then for each face (axis 0-2, side -1
#                then 1) the least the side times the view's depth along the axis (999 scale) for the face to be
#                shown, the eye beyond its plane: side (9 g - p) > 0.48 for the face of a cubie at p along it,
#                999 (0.48 + side p) / 9 rounded down, plus 1
# corners.txt    the corners of a face (4, the cubie's plastic) and of its sticker (8, the octagon), anticlockwise
#                seen from outside along the axis: the lift (1: 0.48, 2: 0.486), then the signs and sizes of the
#                two axes after it (3: 0.42, 4: 0.31 of the unit; 1: 0.48)
# scales.txt     0.48, 0.486, 0.42 and 0.31, in 999ths: the scaled axes' sizes
import math, os
here = os.path.dirname(os.path.abspath(__file__))

def write(name, rows):
    open(os.path.join(here, name), 'w').write(''.join(' '.join(str(v) for v in r) + '\n' for r in rows))

write('sine.txt', [[round(999 * math.sin(math.radians(k)))] for k in range(92)])
xs = [0.50 + k / 100 for k in range(51)]
write('pow10.txt', [[round(999 * x ** 10)] for x in xs])
write('pow24.txt', [[round(999 * x ** 24)] for x in xs])
base = [(0.21, 0.21, 0.23),          # plastic
        (1.00, 0.45, 0.00),          # L orange
        (0.85, 0.07, 0.07),          # R red
        (1.00, 0.88, 0.00),          # D yellow
        (0.95, 0.95, 0.95),          # U white
        (0.05, 0.30, 0.90),          # B blue
        (0.00, 0.70, 0.20)]          # F green
write('materials.txt', [[round(576 * c) for c in m] for m in base])
L = (-0.45, 0.70, 0.75)
n = math.sqrt(sum(v * v for v in L)); L = [v / n for v in L]
H = [L[0], L[1], L[2] + 1]
n = math.sqrt(sum(v * v for v in H)); H = [v / n for v in H]
write('light.txt', [[round(999 * v) for v in L], [round(999 * v) for v in H]])
AX = {'R': (0, 1), 'L': (0, -1), 'U': (1, 1), 'D': (1, -1), 'F': (2, 1), 'B': (2, -1)}
def parse(s):
    return [AX[m[0]] + ({'': 1, "'": -1, '2': 2}[m[1:]],) for m in s.split()]
scramble = parse("R B2 U F' D' F' D F' D F' U2 D' L2 D R2 F'")
solve = parse("F R2 D' L2 D U2 F D' F D' F D F U' B2 R'")
write('moves.txt', scramble + solve)
rows = []
for x in (-1, 0, 1):
    for y in (-1, 0, 1):
        for z in (-1, 0, 1):
            p = (x, y, z)
            for a in range(3):
                if p[a]:
                    nrm = [0, 0, 0]; nrm[a] = p[a]
                    face = 2 * a + (p[a] > 0)
                    rows.append(list(p) + nrm + [face])
write('solved.txt', rows)
rows = []
for x in (-1, 0, 1):
    for y in (-1, 0, 1):
        for z in (-1, 0, 1):
            p = (x, y, z)
            r = list(p) + [round(250 * (x * x + y * y + z * z) / 18)]
            for a in range(3):
                for side in (-1, 1):
                    r.append(math.floor(999 * (0.48 + side * p[a]) / 9) + 1)
            rows.append(r)
write('cubies.txt', rows)
quad = [(1, -1, 1, -1, 1), (1, 1, 1, -1, 1), (1, 1, 1, 1, 1), (1, -1, 1, 1, 1)]
octagon = [(-1, 4, -1, 3), (1, 4, -1, 3), (1, 3, -1, 4), (1, 3, 1, 4), (1, 4, 1, 3), (-1, 4, 1, 3), (-1, 3, 1, 4), (-1, 3, -1, 4)]
write('corners.txt', quad + [(2,) + c for c in octagon])
write('scales.txt', [[480, 486, 420, 310]])
