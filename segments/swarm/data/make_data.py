# Prints the swarm's tables and flight cards from the prototype's numbers (devices/swarm_mid.rs): the cards the host
# deals and the reference reads. Run once: python3 make_data.py (no dependencies). Writes, beside this file:
#   tables.txt  the printed tables: trig, atan, rsq, spec12, spec30, quarter (one per line, the name first)
#   flight.txt  a line per device of the swarm, then a line per credit (columns below)
# Units: lengths in thousandths of the prototype's unit (100 mm), angles in millionths of a turn, a frame a twelfth
# of a second.
import math, os
here = os.path.dirname(os.path.abspath(__file__))
TAU = 2 * math.pi
def r(x): return int(math.floor(x + 0.5))          # half up, as the host's tables are printed
def turns(rad): return r(rad / TAU * 1e6) % 1000000  # an angle in millionths of a turn
def rate(rad_per_s): return r(rad_per_s / 12 / TAU * 1e6)  # radians a second -> millionths of a turn a frame

tables = {
    # 999 sin of a thousandth of a turn, a quarter turn of them
    'trig': [r(999 * math.sin(TAU * k / 1000)) for k in range(251)],
    # atan(k / 2000) in millionths of a turn
    'atan': [r(math.atan(k / 2000) / TAU * 1e6) for k in range(2001)],
    # 1000 / sqrt(2 w / 1000) for w = 0..2000 (w = 1000 less a dot of unit vectors in thousandths; 0 held at w = 1)
    'rsq': [r(1000 / math.sqrt(2 * max(w, 1) / 1000)) for w in range(2001)],
    # a quarter of white times the highlight's power: 63.75 (k / 1000)^12 and ^30, k = 0..999
    'spec12': [r(63.75 * (k / 1000) ** 12) for k in range(1000)],
    'spec30': [r(63.75 * (k / 1000) ** 30) for k in range(1000)],
    # quarter squares, n^2 / 4 rounded down for n = 0..1998, for the machines' products
    'quarter': [n * n // 4 for n in range(1999)],
}
with open(os.path.join(here, 'tables.txt'), 'w') as f:
    for name, v in tables.items():
        f.write(name + ' ' + ' '.join(map(str, v)) + '\n')

# The devices of the swarm, as the prototype flies them: the hero (the drive) first.
# kind: 0 iPad 11-inch, 1 iPad mini, 2 iPhone; metal: 0 silver, 1 black, 2 grey, 3 titanium, 4 blue, 5 desert.
swarm = [
    # kind, metal, segment video, first frame, frames, orbit radius, height, phase, speed, spin, delay
    (0, 2, 'drive/video.rgb', 0, 181, 0.0, 0.0, 0.0, 0.0, 1, 0.00),
    (2, 0, 'opening/video.rgb', 140, 22, 2.5, 0.55, 0.3, 0.55, 1, 0.70),
    (1, 3, 'polys/video.rgb', 200, 174, 2.9, -0.6, 2.4, 0.42, -1, 0.28),
    (2, 1, 'julia/video.rgb', 0, 144, 2.3, -0.95, 4.1, 0.6, 1, 1.26),
    (2, 4, 'roto/video4.rgb', 50, 128, 2.7, 1.05, 1.5, 0.48, -1, 1.68),
    (2, 5, 'ray/video_plasma.rgb', 0, 120, 3.2, 0.2, 3.3, 0.38, 1, 0.84),
    (1, 0, 'deep/video.rgb', 120, 150, 3.4, -0.2, 5.2, 0.33, -1, 1.96),
    (2, 3, 'tunnel/video4.rgb', 0, 136, 2.2, 1.25, 5.9, 0.66, 1, 2.38),
    (2, 1, 'rubik/video.rgb', 0, 73, 3.0, -1.25, 0.9, 0.45, -1, 1.54),
    (2, 0, 'amiga/titled.rgb', 0, 66, 2.6, 0.0, 3.9, 0.52, 1, 2.66),
    (2, 4, 'dsotm/video.rgb', 0, 90, 3.6, 0.95, 2.0, 0.36, -1, 0.56),
    (2, 5, 'tumble/video.rgb', 0, 201, 2.4, -0.45, 1.1, 0.58, 1, 3.08),
]
with open(os.path.join(here, 'flight.txt'), 'w') as f:
    f.write('# The swarm: a line a device. kind metal video first frames spin | delay (thousandths of a frame) |\n'
            '# orbit: phase, rate (a frame), across, up, depth, depth phase, depth rate | bob: phase, rate, size |\n'
            '# wobble about x, y, z: phase, rate |\n'
            '# away: across, up, depth | twirl phase (tenths of a frame)\n')
    for k, (kind, metal, video, first, frames, rad, y, phase, speed, spin, delay) in enumerate(swarm):
        way = -1 if k % 3 == 2 else 1
        if k == 0:
            # the hero: a little figure of eight near the middle, across, up and in depth at 0.5, 1 and 0.45 a second
            orbit = [0, rate(0.5), 350, 0, 150, 250000, rate(0.45)]
            bob = [0, rate(1.0), 180]
        else:
            # round the hero: across sin, in depth cos of the same angle; bobbing a quarter unit
            orbit = [turns(phase), way * rate(speed), r(1350 * rad), r(1200 * y), r(500 * rad), (turns(phase) + 250000) % 1000000, way * rate(speed)]
            bob = [turns(k), rate(0.9 + 0.13 * k), 250]
        wob = [turns(k), rate(1.1 + 0.07 * k), turns(2 * k), rate(0.8 + 0.05 * k), turns(3 * k), rate(0.9)]
        away = [r(1500 * math.sin(1.7 * k)), r(1500 * math.cos(2.3 * k)), -600 * (k % 2)]
        cols = [kind, metal, video, first, frames, spin, r(12000 * delay)] + orbit + bob + wob + away + [84 * k]
        f.write(' '.join(map(str, cols)) + '\n')
    f.write('# The credits: an iPad each. screen file, when it starts to come (tenths of a frame), side\n')
    for name, t0, side in [('credit_saul_x4.rgb', 4.5, 1), ('credit_david_x4.rgb', 8.5, -1), ('credit_ca_x4.rgb', 12.5, 1)]:
        f.write('credit %s %d %d\n' % (name, r(120 * t0), side))
print('tables.txt, flight.txt')
