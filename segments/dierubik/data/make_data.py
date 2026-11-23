# Makes the die and Rubik's cube's data files (run once, plain Python), from the prototype (prototypes/dierubik.py):
#   python make_data.py
# colours.txt   red, green, blue (0..255): the black plastic; the six sticker colours, then the plain cube's six face
#               colours (each by face: 2 axis + 1 if it faces up the axis: -x, +x, -y, +y, -z, +z); the die's ivory;
#               the pips' black
# light.txt     the unit vector toward the light, 999 each part (the prototype's (-0.45, 0.75, -0.5), normalised)
# moves.txt     the scramble (the solve undone: its moves in reverse, each the other way), then the solve, F R2 D' L2 D
#               U2 F D' F D' F D F U' B2 R': axis (0 x, 1 y, 2 z), side (-1, 1) (F is -z, toward the eye), quarter
#               turns clockwise looking at that face (1, -1, 2)
# turns.txt     the frames each move of the solve takes: 5, less one each four moves, at least 3 (60 in all: the die
#               held 3 s took 12 of the solve's 72)
# solved.txt    the solved cube's 54 stickers: cubie place (x, y, z), outward normal (x, y, z), colour (its face)
# cubies.txt    the 27 cubies, x outermost: place (x, y, z)
# pips.txt      per die face (by face, as colours.txt): its pips' places across the face, each on the two other axes in
#               turn (x then y then z, the face's own left out), -1, 0 or 1 times 0.55
# sizes.txt     the parts of the cube (in cube units) the corners are made of, in 500ths: the die's body 1.0, its
#               faces' plates 1.02 out and 0.98 across, its pips 1.05 out, 0.55 apart and 0.17 across at full size;
#               the Rubik's cube's cubies 0.68 apart, 0.33 across, their stickers 0.35 out and 0.28 across, and 0.34
#               (half way between two cubies)
# rates.txt     angles each step adds, in tenths of a degree as quarter turns, tenths and thousandths of a tenth:
#               the tumble (about y, x, z: 1.3, 0.8 and 0.35 radians a second) a frame, and the same for each 0.12 s
#               the pop adds; the roam's four (1/7, 1/4.3, 1/11 and 1/6.5 radians a frame); the wobble's down a row
#               (360 / 40 degrees) and on a frame (-0.5 radians); the pop's (180 / 6 degrees a frame)
import math, os
here = os.path.dirname(os.path.abspath(__file__))

def write(name, rows):
    open(os.path.join(here, name), 'w').write(''.join(' '.join(str(v) for v in r) + '\n' for r in rows))

def face(n):
    a = [i for i in range(3) if n[i]][0]
    return 2 * a + (n[a] > 0)

NORMALS = [(-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0, 0, 1)]
STICKER = {(0, 0, -1): (40, 200, 60), (0, 0, 1): (40, 90, 230), (0, 1, 0): (245, 245, 245), (0, -1, 0): (250, 220, 40),
           (1, 0, 0): (230, 40, 40), (-1, 0, 0): (250, 140, 30)}
TUMBLE_FACES = {(0, 0, -1): (48, 224, 64), (0, 0, 1): (20, 130, 150), (0, 1, 0): (224, 64, 224),
                (0, -1, 0): (135, 117, 18), (1, 0, 0): (240, 48, 48), (-1, 0, 0): (60, 90, 230)}
IVORY, PIP, PLASTIC = (242, 236, 220), (25, 25, 30), (20, 20, 22)
write('colours.txt', [PLASTIC] + [STICKER[n] for n in NORMALS] + [TUMBLE_FACES[n] for n in NORMALS] + [IVORY, PIP])

L = (-0.45, 0.75, -0.5)
n = math.sqrt(sum(v * v for v in L))
write('light.txt', [[round(999 * v / n) for v in L]])

FACE = {"F": (2, -1), "B": (2, 1), "U": (1, 1), "D": (1, -1), "R": (0, 1), "L": (0, -1)}
SOLVE = "F R2 D' L2 D U2 F D' F D' F D F U' B2 R'".split()
def move(m, inverse=False):
    q = 2 if m.endswith("2") else (-1 if m.endswith("'") else 1)
    return FACE[m[0]] + ((-q if q != 2 else 2) if inverse else q,)
write('moves.txt', [move(m, True) for m in reversed(SOLVE)] + [move(m) for m in SOLVE])
write('turns.txt', [[max(3, 5 - i // 4) for i in range(len(SOLVE))]])

rows, cubies = [], []
for x in (-1, 0, 1):
    for y in (-1, 0, 1):
        for z in (-1, 0, 1):
            p = (x, y, z)
            cubies.append(p)
            for a in range(3):
                if p[a]:
                    nrm = [0, 0, 0]; nrm[a] = p[a]
                    rows.append(list(p) + nrm + [face(nrm)])
write('solved.txt', rows)
write('cubies.txt', cubies)

PIPS = {1: [(0, 0)], 2: [(-1, -1), (1, 1)], 3: [(-1, -1), (0, 0), (1, 1)], 4: [(-1, -1), (-1, 1), (1, -1), (1, 1)],
        5: [(-1, -1), (-1, 1), (0, 0), (1, -1), (1, 1)], 6: [(-1, -1), (-1, 0), (-1, 1), (1, -1), (1, 0), (1, 1)]}
DIE = {(0, 0, -1): 1, (0, 0, 1): 6, (0, 1, 0): 2, (0, -1, 0): 5, (1, 0, 0): 3, (-1, 0, 0): 4}
rows = []
for nrm in NORMALS:
    # the prototype's two axes across a face: u = x unless the face is along x (then z), v = n x u
    u = (1, 0, 0) if abs(nrm[0]) < 0.5 else (0, 0, 1)
    v = (nrm[1] * u[2] - nrm[2] * u[1], nrm[2] * u[0] - nrm[0] * u[2], nrm[0] * u[1] - nrm[1] * u[0])
    a = [i for i in range(3) if nrm[i]][0]
    row = []
    for s, t in PIPS[DIE[nrm]]:
        place = [s * u[k] + t * v[k] for k in range(3)]
        row += [place[k] for k in range(3) if k != a]
    rows.append(row)
write('pips.txt', rows)
write('sizes.txt', [[500, 510, 490, 525, 275, 85], [340, 165, 175, 140, 170]])

def rate(v):
    """tenths of a degree as quarter turns, tenths and thousandths of a tenth"""
    th = round(v * 1000)
    return [th // 900000, th // 1000 % 900, th % 1000]
tenths = lambda rad: rad * 1800 / math.pi
rows = [rate(tenths(r / 12)) for r in (1.3, 0.8, 0.35)] + [rate(tenths(r * 0.12)) for r in (1.3, 0.8, 0.35)]
rows += [rate(tenths(1 / d)) for d in (7, 4.3, 11, 6.5)]
rows += [rate(900 / 10), rate(3600 - tenths(0.5)), rate(300)]
write('rates.txt', rows)
