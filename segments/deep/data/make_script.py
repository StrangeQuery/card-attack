# The dive's script's printed numbers (script.txt here is read by segments/deep/script.rs): the canyon's lips the
# diver swims to, and each creature's fixed numbers, made from the prototype's random numbers (its hash of a
# creature's number and a salt, 0..65535). Places in millionths of a unit.
#
#     python3 make_script.py > script.txt
#
# Lines: "marks" then each lip's z and y; "jelly" per jellyfish k: its place on the path (a sample, 48 a second of
# story time), its place across and up from the path, how far its bell is turned (64ths), its kind (0 blue, 1 violet),
# its eight tentacles' lengths (tenths); "kelp" per clump k: its place along (before the forest goes round) and across,
# across in whole units and hundredths, then per stalk its lean (15ths) and height (tenths) and per blade on it which
# side it leans (1 or -1) and its length (50ths); "grass" per tuft k: along, across, across in units and hundredths,
# its five blades' heights (tenths); "fish" per fish: its place about its school's leader, across, up and along.
import math

M = 1_000_000
TURN = 1_000_000_000

def rand(a, b):
    return ((a * 73856093 ^ b * 19349663) * 2654435761) & 0xffff

def rnd(n, d):
    return (2 * n + d) // (2 * d) if n >= 0 else -((-2 * n + d) // (2 * d))

RAD1 = round(1.0 / (2 * math.pi) * 1e9)

out = []
marks, z, h = [0, 8 * M], 115, -4
for k in range(40):
    marks += [z * M, (h + 12) * M]
    h -= 18 + 9 * k
    z += 24 + max(36 - 4 * k, 20)
out.append("marks " + " ".join(map(str, marks)))

for k in range(60):
    side = 1 if k % 2 == 0 else -1
    v = [((6960 + 768 * k) * 65536 + 360 * rand(k, 61)) // (5 * 65536),
         side * (5 * M // 2 + 5 * M * rand(k, 62) // 65536),
         -4 * M + 7 * M * rand(k, 63) // 65536,
         rnd(64 * k * RAD1, TURN) % 64,
         int(10 * rand(k, 64) >= 7 * 65536)]
    v += [rnd(22 * 65536 + 9 * rand(tn, k), 65536) for tn in range(8)]
    out.append("jelly " + " ".join(map(str, v)))

def across(x):
    h = rnd(x, 10_000)
    return [h // 100, h % 100]

for k in range(14):
    kx = 34 * M * (rand(k, 9) - 32768) // 65536
    v = [45 * M * rand(k, 11) // 65536, kx] + across(kx)
    for st in range(3):
        v += [rnd(9 * (st - 1) * 65536 + 6 * (rand(k, 20 + st) - 32768), 65536), rnd(120 * 65536 + 100 * rand(k * 3 + st, 13), 65536)]
        for i in range(1, 16):
            v += [1 if (i + st) % 2 == 0 else -1, rnd((11 * 65536 + 5 * rand(k * 7 + st, i)) * (3021 - 70 * i), 65536 * 600)]
    out.append("kelp " + " ".join(map(str, v)))

for k in range(60):
    gx = 24 * M * (rand(k, 33) - 32768) // 65536
    v = [20 * M * rand(k, 31) // 65536, gx] + across(gx) + [rnd(6 * 65536 + 4 * rand(k, 40 + b), 65536) for b in range(5)]
    out.append("grass " + " ".join(map(str, v)))

for sk, n in ((0, 40), (1, 24)):
    for k in range(n):
        q = k + 100 * sk
        out.append("fish " + " ".join(str(s * M * (rand(q, c) - 32768) // 65536) for c, s in ((1, 9), (2, 3), (3, 6))))

print("\n".join(out))
