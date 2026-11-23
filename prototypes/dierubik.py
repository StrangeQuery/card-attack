"""Preview: the tumbling cube, carried on. From where the tumble's cube is small and far (its frame 147), it comes
toward us still tumbling, turns into a die (the faces go ivory, the pips fizz in), splits into a scrambled Rubik's
cube and, still tumbling, plays the rubik segment's real 16-move solve, the last turn snapping in; a pop and a spin,
solved. 192x96 like the tumble (drawn at 384x192, each pixel the mean of four). Black where nothing is, for the BBS.

    frames()      the sequence, a list of 96x192x3 uint8
"""
import math

import numpy as np

import mtv_aha as M
from mtv import trans

FPS = 12
RW, RH = 384, 192
SOLVE = "F R2 D' L2 D U2 F D' F D' F D F U' B2 R'".split()   # the rubik segment's solve (its scramble undone)
FACE = {"F": (0, 0, -1), "B": (0, 0, 1), "U": (0, 1, 0), "D": (0, -1, 0), "R": (1, 0, 0), "L": (-1, 0, 0)}
STICKER = {(0, 0, -1): (40, 200, 60), (0, 0, 1): (40, 90, 230), (0, 1, 0): (245, 245, 245), (0, -1, 0): (250, 220, 40),
           (1, 0, 0): (230, 40, 40), (-1, 0, 0): (250, 140, 30)}
TUMBLE_FACES = {(0, 0, -1): (48, 224, 64), (0, 0, 1): (20, 130, 150), (0, 1, 0): (224, 64, 224),
                (0, -1, 0): (135, 117, 18), (1, 0, 0): (240, 48, 48), (-1, 0, 0): (60, 90, 230)}
IVORY, PIP, PLASTIC = (242, 236, 220), (25, 25, 30), (20, 20, 22)
PIPS = {1: [(0, 0)], 2: [(-1, -1), (1, 1)], 3: [(-1, -1), (0, 0), (1, 1)], 4: [(-1, -1), (-1, 1), (1, -1), (1, 1)],
        5: [(-1, -1), (-1, 1), (0, 0), (1, -1), (1, 1)], 6: [(-1, -1), (-1, 0), (-1, 1), (1, -1), (1, 0), (1, 1)]}
DIE = {(0, 0, -1): 1, (0, 0, 1): 6, (0, 1, 0): 2, (0, -1, 0): 5, (1, 0, 0): 3, (-1, 0, 0): 4}

def rot(axis, a):
    """Rotation by a about a unit axis (Rodrigues), 4x4."""
    x, y, z = axis
    c, s, t = math.cos(a), math.sin(a), 1 - math.cos(a)
    m = np.eye(4)
    m[:3, :3] = [[t * x * x + c, t * x * y - s * z, t * x * z + s * y],
                 [t * x * y + s * z, t * y * y + c, t * y * z - s * x],
                 [t * x * z - s * y, t * y * z + s * x, t * z * z + c]]
    return m

def tumble(t):
    return rot((0, 1, 0), 1.3 * t) @ rot((1, 0, 0), 0.8 * t) @ rot((0, 0, 1), 0.35 * t)

def face_frame(n):
    """Two axes across a face with normal n."""
    n = np.array(n, float)
    u = np.array([1.0, 0, 0]) if abs(n[0]) < 0.5 else np.array([0, 0, 1.0])
    v = np.cross(n, u)
    return u, v

# ---- the Rubik's cube: cubies with a place and a turn --------------------------------------------------------------

def solved():
    cubies = []
    for x in (-1, 0, 1):
        for y in (-1, 0, 1):
            for z in (-1, 0, 1):
                if (x, y, z) == (0, 0, 0):
                    continue
                stickers = [(n, STICKER[n]) for n in STICKER if np.dot(n, (x, y, z)) == 1]
                cubies.append({"p": np.array([x, y, z], float), "o": np.eye(4), "s": stickers})
    return cubies

def move_of(m):
    n = FACE[m[0]]
    turns = 2 if m.endswith("2") else (-1 if m.endswith("'") else 1)
    return np.array(n, float), -turns * math.pi / 2       # clockwise from outside

def apply(cubies, m, inverse=False):
    n, a = move_of(m)
    if inverse:
        a = -a
    r = rot(n, a)
    for c in cubies:
        if round(float(np.dot(c["p"], n))) == 1:
            c["p"] = np.round(r[:3, :3] @ c["p"])
            c["o"] = r @ c["o"]

def scrambled():
    cubies = solved()
    for m in reversed(SOLVE):
        apply(cubies, m, inverse=True)
    return cubies

SP = 0.68
def rubik_scene(cubies, whole, turning=None, scale=1.0):
    """The cube at world transform whole; turning = (move, fraction) for the slice in motion."""
    s = M.Scene()
    if turning:
        n, a = move_of(turning[0])
        part = rot(n, a * turning[1])
    for c in cubies:
        m = whole @ np.diag([scale, scale, scale, 1])
        if turning and round(float(np.dot(c["p"], move_of(turning[0])[0]))) == 1:
            m = m @ part
        m = m @ trans(*(c["p"] * SP)) @ c["o"]
        s.box(m, (0.66, 0.66, 0.66), PLASTIC)
        for nrm, col in c["s"]:
            sz = [0.56, 0.56, 0.56]
            sz[int(np.argmax(np.abs(nrm)))] = 0.03
            s.box(m @ trans(*(np.array(nrm) * 0.335)), tuple(sz), col)
    return s

def die_scene(whole, k_colour, k_pips):
    """The plain cube going to a die: each face from its tumble colour to ivory by k_colour; pips by k_pips."""
    s = M.Scene()
    s.box(whole, (2.0, 2.0, 2.0), PLASTIC)
    for n, col in TUMBLE_FACES.items():
        c = tuple(int(a + (b - a) * k_colour) for a, b in zip(col, IVORY))
        sz = [1.96, 1.96, 1.96]
        sz[int(np.argmax(np.abs(n)))] = 0.04
        s.box(whole @ trans(*(np.array(n) * 1.0)), tuple(sz), c)
        if k_pips > 0:
            u, v = face_frame(n)
            for a, b in PIPS[DIE[n]]:
                p = np.array(n) * 1.03 + u * a * 0.55 + v * b * 0.55
                ps = 0.34 * k_pips
                sz = [ps, ps, ps]
                sz[int(np.argmax(np.abs(n)))] = 0.04
                s.box(whole @ trans(*p), tuple(sz), PIP)
    return s

# ---- the shot -------------------------------------------------------------------------------------------------------

EYE = np.array([0.0, 0.0, -10.0])
BASIS = M.look(EYE, np.array([0.0, 0.0, 0.0]))
VFOV = math.radians(40)
HFOV = 2 * math.atan(math.tan(VFOV / 2) * RW / RH)

def at(depth, sx, sy):
    """The world point seen at film pixel (sx, sy) of 192x96, depth units in front of the eye."""
    f = 48 / math.tan(VFOV / 2)
    return np.array([(sx - 96) / f * depth, (48 - sy) / f * depth, -10 + depth])

def render(scene):
    img = M.raster(scene.tris, EYE, BASIS, HFOV, RW, RH, {})[0]
    return np.clip(img.reshape(96, 2, 192, 2, 3).mean((1, 3)), 0, 255).astype(np.uint8)

def ease(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)

def frames(tail=0):
    out = []
    start, end = at(13.0, 83.5, 43.5), at(395 / 70, 96, 48)          # (5.64: an edge of the held die within a card)
    t = 0.0
    def place(k, n):
        p = start + (end - start) * ease(k / n)
        return p
    def whole(p, t, scale=1.0):
        return trans(*p) @ tumble(t) @ np.diag([scale, scale, scale, 1])
    # coming toward us, tumbling (18), going to a die from its sixth frame (8), the die held (32): whole for 3 s
    for k in range(18):
        j = k - 6
        colour, pips = (0.0, 0.0) if j < 0 else ((j + 1) / 8, max(0.0, (j - 3) / 4)) if j < 8 else (1.0, 1.0)
        out.append(render(die_scene(whole(place(k, 18), t), colour, pips))); t += 1 / FPS
    p = end
    for k in range(32):
        out.append(render(die_scene(whole(p, t), 1.0, 1.0))); t += 1 / FPS
    # the die splits into the scrambled Rubik's cube (6, crossfading), then the solve, still tumbling
    cubies = scrambled()
    for k in range(6):
        u = (k + 1) / 7
        a = render(die_scene(whole(p, t), 1.0, 1.0)).astype(np.float32)
        b = render(rubik_scene(cubies, whole(p, t, 1.0 + 0.08 * (1 - u)))).astype(np.float32)
        out.append((a * (1 - u) + b * u).astype(np.uint8)); t += 1 / FPS
    for i, m in enumerate(SOLVE):
        n = max(3, 5 - i // 4)                            # each turn a little quicker than the last
        for k in range(n):
            out.append(render(rubik_scene(cubies, whole(p, t), (m, ease((k + 1) / n))))); t += 1 / FPS
        apply(cubies, m)
    for k in range(14):                                   # solved: a pop and a spin
        pop = 1 + 0.06 * math.sin(math.pi * min(1.0, k / 6))
        out.append(render(rubik_scene(cubies, whole(p, t + 0.12 * k, pop)))); t += 1 / FPS
    t += 0.12 * 14
    for k in range(tail):                                 # and on, tumbling, drifting about, off into the distance and back
        z = 2.6 * (1 - math.cos(k / 7)) + 1.2 * (1 - math.cos(k / 4.3))
        q = p + np.array([(0.5 + 0.55 * z) * math.sin(k / 11), (0.3 + 0.3 * z) * math.sin(k / 6.5), z])
        out.append(render(rubik_scene(cubies, whole(q, t)))); t += 1 / FPS
    return out

if __name__ == "__main__":
    import subprocess, sys
    fs = frames()
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "192x96",
                           "-r", str(FPS), "-i", "-", "-vf", "scale=1536:768:flags=neighbor", "-c:v", "libx264",
                           "-pix_fmt", "yuv420p", "-crf", "16", "-movflags", "+faststart", sys.argv[1]], stdin=subprocess.PIPE)
    for f in fs:
        ff.stdin.write(f.tobytes())
    ff.stdin.close(); ff.wait()
    print(len(fs), "frames,", round(len(fs) / FPS, 1), "s")
