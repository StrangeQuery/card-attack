# Prints the drive's data cards, once: the street's plan, the turn table, the city's blocks, the traffic, the
# script and the car's picture. The numbers come from the prototype (city3.rs, not in this repository) at its own scale:
# 1 unit = a prototype unit, positions along the street in quarters (q), across it in eighths (e).
# python3 make_data.py   (writes the .txt files beside it)
import math, os
here = os.path.dirname(os.path.abspath(__file__))
def put(name, lines): open(os.path.join(here, name), 'w').write('\n'.join(lines) + '\n')
def smooth(e0, e1, x):
    t = min(max((x - e0) / (e1 - e0), 0.0), 1.0)
    return t * t * (3 - 2 * t)
def hash2(a, b): return ((((a * 73856093) ^ (b * 19349663)) * 2654435761) & 0xffff) / 65536.0

# The street's middle, across, every 2 units along: straight, bending left, then right, then straight again.
def road(z): return -26 * smooth(70, 150, z) + 26 * smooth(170, 250, z)
put('road.txt', [str(round(8 * road(2 * i))) for i in range(300)])

# The camera's turn for a slope of d eighths across in 96 (12 units along): cosine and sine, times 400.
put('turn.txt', ['%d %d' % (round(400 * 96 / math.hypot(96, d)), round(400 * d / math.hypot(96, d))) for d in range(81)])

# The blocks, 30 units apart (26 long, a 4-unit gap): per block, left then right, the setback from the street's
# middle (eighths), its height in half-unit bands (6 a floor of 3 units, then 2 for a 1-unit parapet), the wall's
# tone (0 or 1) and where its windows start in the window texture (32 a column, plus a floor to start at).
blocks = []
for b in range(22):
    row = []
    for side in (-1, 1):
        front = 9 + 4 * hash2(b, side * 3)
        height = 8 + 36 * hash2(b, side)
        floors = min(14, max(2, round((height - 1) / 3)))
        tone = 0 if hash2(b, 7 + side) < 0.5 else 1
        row += [round(8 * front), 6 * floors + 2, tone, 32 * int(30 * hash2(b * 977 + 13, 11 + side)) + int(16 * hash2(b, 23 + side))]
    blocks.append(' '.join(map(str, row)))
put('blocks.txt', blocks)

# The window texture: 30 columns of 32 floors, a card each, column by column: a window lit (0.42 of them) in its
# floor's colour (5-8, warm to cool), else unlit (4).
put('windows.txt', [str(5 + int(4 * hash2(k, 99)) if hash2(c * 977 + 5, k * 31 + 1) < 0.42 else 4) for c in range(30) for k in range(32)])

# Traffic. We go 27 units a second (9 q a frame), the slow cars in our lane 15 (5 q), the oncoming ones 21 (7 q).
SPEED, SLOW, ONCOMING = 27.0, 15.0, 21.0
slow = [40.0, 95.0, 150.0, 205.0]
def car_x(t):   # our car: in its lane, out round each slow car it catches up with, a slight wander
    z = SPEED * t
    out = max(smooth(26, 14, s0 + SLOW * t - z) * smooth(-10, -2, s0 + SLOW * t - z) for s0 in slow)
    return 2.5 - out * 4 + 0.4 * math.sin(t * 1.7)
# oncoming cars meet our car (7 units ahead of the camera) at o0 - ONCOMING t = SPEED t + 7: each one later until
# we're back in our own lane for half a second either side of that, and 10 frames after the one before; kept as the frame they meet us
meet = []
for o in [120.0, 190.0, 230.0, 300.0, 340.0, 420.0, 480.0]:
    while any(car_x((o - 7) / (SPEED + ONCOMING) + k / 12) < 1.6 for k in range(-6, 7)) or \
            (meet and 12 * (o - 7) / (SPEED + ONCOMING) < meet[-1] + 10): o += 6   # and not on the last one's heels
    meet.append(round(12 * (o - 7) / (SPEED + ONCOMING)))
COLOURS = 5
cars = [(0, round(4 * s0), k % COLOURS) for k, s0 in enumerate(slow)] + [(1, m, (k + 2) % COLOURS) for k, m in enumerate(meet)]

# The script, a line a frame: the camera's place across the street (eighths from its middle; it follows our car a
# beat behind), our car's left edge on screen (quarters of a pixel) and its top (half pixels; it bobs).
script = []
for f in range(144):
    t = f / 12
    lane = car_x(t - 0.25)
    lean = car_x(t) - lane
    script.append('%d %d %d' % (round(8 * lane), round(4 * (96 - 27 + lean * 18)), 146 + round(math.sin(t * 9))))
put('script.txt', script)

# A car from behind and from the front, 36 x 14: . nothing, r body, d dark, w glass, l lights (tail or head), k tyre,
# p plate; three to a card, as digits 0-6.
REAR = ["..........dddddddddddddddd..........", "........ddwwwwwwwwwwwwwwwwdd........", ".......dwwwwwwwwwwwwwwwwwwwwd.......",
    "......dwwwwwwwwwwwwwwwwwwwwwwd......", "...rrrrrrrrrrrrrrrrrrrrrrrrrrrrrr...", "..rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr..",
    ".rllllllrrrrrrrrrrrrrrrrrrrrllllllr.", ".rllllllrrrrrrrrrrrrrrrrrrrrllllllr.", ".rrrrrrrrrrrrrrppppppppprrrrrrrrrrr.",
    ".ddddddddddddddppppppppdddddddddddd.", ".dddddddddddddddddddddddddddddddddd.", ".kkkkkkk......................kkkkkk",
    ".kkkkkkk......................kkkkkk", "..kkkkk........................kkkk."]
FRONT = REAR[:6] + [".rllllllrrrrddddddddddddddrrllllllr."] * 2 + [".rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr.", ".dddddddddddddddddddddddddddddddddd."] + REAR[10:]
SYM = '.rdwlkp'
def cards(art): return [' '.join(str(100 * SYM.index(r[i]) + 10 * SYM.index(r[i + 1]) + SYM.index(r[i + 2])) for i in range(0, 36, 3)) for r in art]
put('car.txt', cards(REAR) + cards(FRONT))

# The jet seen from the front, 36 x 14, its half wingspan 18 columns, from the fin's tip (9/16 of that above its
# middle) down 7/8 of it: 1 grey, 2 the body (the strobe), 3 and 4 the left and right wingtips.
def jet(u, v):
    ax, ay = (u + 0.5) / 18 - 1, -9 / 16 + (v + 0.5) / 14 * 7 / 8
    wing = abs(ax) < 1 and abs(ay - abs(ax) * 0.12) < 0.18
    body = abs(ax) < 0.18 and abs(ay) < 0.3
    fin = abs(ax) < 0.08 and -0.55 < ay < 0
    return (3 if ax < 0 else 4) if wing and abs(ax) > 0.9 else 2 if body else 1 if wing or fin else 0
put('jet.txt', [' '.join(str(100 * jet(i, v) + 10 * jet(i + 1, v) + jet(i + 2, v)) for i in range(0, 36, 3)) for v in range(14)])

# Colour: a card's three digits are red, green and blue, each a step 0-9 of its channel's ten levels (the paint
# machine's table); denser in the dark, where the night is.
LEVELS = [[0, 12, 26, 38, 52, 70, 100, 150, 205, 255], [0, 12, 26, 36, 48, 62, 90, 140, 205, 255], [0, 16, 28, 40, 54, 70, 100, 150, 205, 255]]
put('levels.txt', [' '.join(map(str, l)) for l in LEVELS])
def code(rgb): return 100 * min(range(10), key=lambda i: abs(LEVELS[0][i] - rgb[0])) + 10 * min(range(10), key=lambda i: abs(LEVELS[1][i] - rgb[1])) + min(range(10), key=lambda i: abs(LEVELS[2][i] - rgb[2]))
def mix(a, b, k): return [a[i] + (b[i] - a[i]) * k for i in range(3)]
def shade(c, k): return [v * k for v in c]
wall = [mix([30, 30, 44], [64, 52, 60], 0.25), mix([30, 30, 44], [64, 52, 60], 0.75)]
warm, cool = [255, 214, 120], [180, 220, 255]
front = wall + [mix(w, [120, 110, 120], 0.5) for w in wall] + [[14, 16, 26]] + [mix(warm, cool, k / 3) for k in range(4)]
road_, pool = [28, 28, 34], [120, 96, 60]
# the materials the street machines punch, 0-23: the walls' (lit face, then end walls in shade), then the ground's
materials = front + [shade(c, 0.62) for c in front] + [mix(road_, pool, k) for k in (0, 0.2, 0.4, 0.6)] + \
    [[200, 190, 120], [150, 150, 150], [56, 54, 58], [34, 32, 38]]
named = [('sky_low', [44, 26, 62]), ('sky_top', [6, 8, 22]), ('post', [70, 70, 76]), ('lamp', [255, 230, 170]),
    ('glare', [255, 250, 220]), ('glass', [26, 30, 44]), ('tail', [255, 50, 50]), ('head', [255, 250, 220]), ('head2', [255, 240, 200]),
    ('tyre', [12, 12, 14]), ('plate', [220, 220, 200]), ('jet', [70, 72, 92]), ('tip_red', [255, 50, 50]), ('tip_green', [60, 255, 90]),
    ('strobe', [255, 255, 255]), ('caption', [240, 240, 240]), ('black', [0, 0, 0]), ('white', [255, 255, 255]),
    ('red', [176, 26, 30]), ('red_dark', shade([176, 26, 30], 0.45))]
bodies = [[40, 90, 180], [220, 200, 60], [230, 230, 230], [60, 160, 90], [150, 60, 170], [176, 26, 30]]   # the traffic's five, then ours
# The traffic: slow (0, how far ahead at frame 0, quarters) or oncoming (1, the frame it meets us), its lane
# (eighths from the middle), its picture (0 from behind, 1 from the front) and its body's colours, light and dark.
put('cars.txt', ['%d %d %d %d %d %d' % (kind, v, 20 if kind == 0 else -21, kind, code(bodies[c]), code(shade(bodies[c], 0.45))) for kind, v, c in cars])
put('colours.txt', ['materials ' + ' '.join(str(code(c)) for c in materials)] + ['%s %d' % (n, code(c)) for n, c in named] +
    ['bodies ' + ' '.join(str(code(c)) for c in bodies), 'darks ' + ' '.join(str(code(shade(c, 0.45))) for c in bodies)])

# The cover: each channel's ten levels lifted for print, 255 (level / 255)^0.55; and the cover's own levels, 0-3
# the poster's four steps (24, 96, 168, 240), 4 black, 5 the panels' white frames, 6 the cover's ground (8, 8, 10),
# 7 the flash's white.
put('lift.txt', [' '.join(str(round(255 * (v / 255) ** 0.55)) for v in l) for l in LEVELS])
put('cover.txt', [' '.join(map(str, [24, 96, 168, 240, 0, 250, g, 255, 0, 0])) for g in (8, 8, 10)])
