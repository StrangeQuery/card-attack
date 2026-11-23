"""The plasma (segments/plasma/formula.rs), ported line for line to Python: plain loops, then NumPy. Run from the repo's
root (docs/BENCHMARK.md)."""
import math, re, sys, time
import numpy as np
SRC = open("demokit/src/lib.rs").read()
def table(name):
    m = re.search(name + r"[^=]*=\s*\[([^\]]*)\]", SRC)
    return [int(v) for v in re.findall(r"-?\d+", m.group(1))]
W96, W64 = table("PL_WAVE96"), table("PL_WAVE64")
def wave(k, n):
    q = W96 if n == 96 else W64
    k %= n
    h, sign = (k, 1) if k < n // 2 else (k - n // 2, -1)
    j = n // 2 - h if h > n // 4 else h
    return 32 + sign * q[j]
def ring_distance(u, v):
    n = u * u + (v - 48) * (v - 48)
    k = 0
    while (k + 1) * (k + 1) <= 4 * n + 1: k += 1
    return (k + 1) // 2
def plasma(u, v, tf):
    s = wave(ring_distance(u, v) - 2 * tf, 64) + wave(u + 6 * tf, 96) + wave(v + 2 * tf, 96) + wave(u - v + 77 + 2 * tf, 96)
    return (s + 8 * tf) % 256
rows = [list(map(int, l.split())) for l in open("segments/plasma/data/ring.txt")]
keys = [rows[6 * k][:3] for k in range(8)]
def colour(q):
    k, j = q // 32, q % 32
    a, b = keys[k], keys[(k + 1) % 8]
    return [a[c] + (b[c] - a[c]) * j // 32 for c in range(3)]
def frame(tf):
    out = bytearray()
    for y in range(96):
        for x in range(192):
            out += bytes(colour(plasma(x, y, tf)))
    return out
N = 5
t = time.perf_counter()
for f in range(N): frame(f)
py = (time.perf_counter() - t) / N
# NumPy: the whole frame at once (the ring distance by a float square root: NumPy has no loop to keep it exact)
yy, xx = np.mgrid[0:96, 0:192]
w96, w64 = np.array([wave(k, 96) for k in range(96)]), np.array([wave(k, 64) for k in range(64)])
keysa = np.array(keys)
def frame_np(tf):
    n = xx * xx + (yy - 48) ** 2
    rd = ((np.sqrt(4 * n + 1).astype(int)) + 1) // 2
    s = w64[(rd - 2 * tf) % 64] + w96[(xx + 6 * tf) % 96] + w96[(yy + 2 * tf) % 96] + w96[(xx - yy + 77 + 2 * tf) % 96]
    q = (s + 8 * tf) % 256
    k, j = q // 32, q % 32
    a, b = keysa[k], keysa[(k + 1) % 8]
    return (a + (b - a) * j[..., None] // 32).astype(np.uint8)
N2 = 200
t = time.perf_counter()
for f in range(N2): frame_np(f)
npy = (time.perf_counter() - t) / N2
print(f"plain Python {py*1e3:.0f} ms a frame; NumPy {npy*1e3:.2f} ms a frame")
