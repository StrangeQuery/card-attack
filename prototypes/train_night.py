"""Design prototype (the night extension): the ride carries straight on from train.py's last frame, one
continuous shot. The sun touches the horizon and goes down, the sky runs through sunset and dusk to night, the
lights come on in the town and the train as it darkens; the train brakes into a station after dark and stops in the
cab's view beside the big LED board: TONIGHT: DEMOSPLASH 0x10. Then the camera moves forward out of the cab,
through the windscreen, finds a drone hovering just outside and chases it, the drone leading, up out of the station,
over the town and on toward the stadium, until the audience design's own camera takes over (its stadium.py,
branch audience-design): the drone peels off, the picture dissolves into their render, and their film runs on.

    0-119    the cab: full speed, the sunset, braking into the station; stopped at frame 120
    120-131  stopped, the board in view
    132-145  out through the windscreen, the drone just ahead
    146-     the chase, accelerating to their 74 m/s; the last frames dissolve into their render, at their camera(0)
    then     their film's first two seconds (their frames, appended for the preview)

It uses train.py's renderer through its hooks (with them unset, train.py renders exactly as before), the models of
models/build_models.py and models/build_night_models.py, and the audience design's stadium (its geometry, its
lights' positions and sweeps, and for the dissolve its renderer) from prototypes/stadium.py (or the folder named by AUDIENCE).

    python train_night.py OUT.mp4 [--size 1536x768] [--jobs 12]
    python train_night.py --stills K,K,K OUTDIR [--size 768x384]
"""
import json
import math
import os
import subprocess
import sys

import numpy as np

import callouts as CO
import train as T


def _work(*parts):
    """A path in the source files' folder (models, footage, caches; not in this repository), named by CARD_ATTACK_WORK."""
    w = os.environ.get("CARD_ATTACK_WORK")
    if not w:
        sys.exit("set CARD_ATTACK_WORK to the folder holding the source files (they are not in this repository)")
    return os.path.join(os.path.expanduser(w), *parts)


AUDIENCE = os.path.expanduser(os.environ.get("AUDIENCE", os.path.dirname(os.path.abspath(__file__))))
AUWORK = _work("audience", "")
Z0 = T.NFRAMES * T.SPEED                                  # the ride's last frame's cab, plus a frame
BX = float(T.bend(5000.0))                                # the S-curve's shift: everything here is 70 m over
X2, MID, PH = T.X2, T.MID, 1.25

# ---- the timeline -------------------------------------------------------------------------------------------------

CRUISE, DECEL, HOLD, PUSH = 24, 96, 12, 14
F_STOP = CRUISE + DECEL                                   # 120: stopped
F_GO = F_STOP + 10                                        # 130: the dissolve from the cab to the platform begins
F_VIEW = F_GO + 12                                        # 142: on the platform (the dissolve done)
F_BTN = F_VIEW - 8                                        # the drone swooping in (no box now: straight to the drone)
F_CRACK, F_SMASH, F_KEY = 10 ** 6, -10 ** 6, 10 ** 6      # (the break-glass box, cut: these never come)
F_PUSH = F_GO
F_CHASE = F_BTN + 18                                      # the drone's here; the flight carries on from the platform
HAND_V = 74.0 / T.FPS                                     # their camera's speed at the hand-off, metres a frame
XFADE = 24                                                # the join with the ride: the new world dissolves in
DISSOLVE = 44                                             # into the bowl: their render (the crowd, lit) coming up
                                                          # at us once the airship's out of the frame
APPEND = 24                                               # their film's first frames, for the preview

def speed(f):
    if f < CRUISE:
        return T.SPEED
    s = min(1.0, (f - CRUISE) / DECEL)
    return T.SPEED * (1 - s) ** 1.15                       # braking hard, easing at the very end

_ZTAB = np.concatenate([[0.0], np.cumsum([speed(k + 0.5) for k in range(F_STOP + 2)])]) + Z0

def za(f):
    return float(np.interp(min(f, F_STOP), np.arange(len(_ZTAB)), _ZTAB))

ZS = round(za(F_STOP), 2)                                 # the cab's eye when stopped
TIP = ZS + 6.4                                            # our nose's tip (the eye sits under the windscreen)
BOARD_Z = ZS + 14.0
PL1 = ZS + 42.0                                           # the canopies end a little past the board
PL0 = PL1 - 200.0
TOWN0 = PL0 - 230.0
AUD_C = np.array([-258.0, T.LAND, ZS + 20.0])             # the stadium's centre, their origin: film = C + (X, Z, Y):
                                                          # back from the station down an avenue: its face 120 m from
                                                          # the post (two seconds for the drone)

def to_film(P):
    """The audience frame (X right, Y to the stage, Z up) into the film's unbent frame."""
    P = np.asarray(P, float)
    return np.stack([AUD_C[0] + P[..., 0], AUD_C[1] + P[..., 2], AUD_C[2] + P[..., 1]], -1)

def to_aud(P, bent=True):
    P = np.asarray(P, float)
    x = P[..., 0] - (BX if bent else 0.0)
    return np.stack([x - AUD_C[0], P[..., 2] - AUD_C[2], P[..., 1] - AUD_C[1]], -1)


# ---- the light: the sun going down, the dark coming ---------------------------------------------------------------

SUN0 = T.DUSK.sun.copy()
AZ = np.array([SUN0[0], 0.0, SUN0[2]]) / np.linalg.norm([SUN0[0], SUN0[2]])
BASE = dict(amb=T.DUSK.amb.copy(), bounce=T.DUSK.bounce.copy(), haze=T.DUSK.haze.copy(), fog=T.DUSK.fog)

def sun_elevation(f):
    return math.asin(SUN0[1]) - 0.0028 * min(f, F_STOP) - 0.002 * max(0, f - F_STOP)

def smooth(a, b, x):
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)

def ease(x):
    x = min(max(x, 0.0), 1.0)
    return x * x * (3 - 2 * x)

STATE = {"f": 0.0}

def set_light(f):
    """The sky and the light for frame f of the extension: the sun lower each frame, its light redder and gone."""
    el = sun_elevation(f)
    night = smooth(0.0, -0.16, el)
    k = smooth(-0.015, 0.05, el)
    kr = smooth(0.0, 0.1, el)
    d = AZ * math.cos(el); d[1] = math.sin(el)
    E = T.DUSK
    E.sun = d
    E.sun_col = np.array([2.6, 1.05 * (0.45 + 0.55 * kr), 0.35 * kr]) * k
    na = np.array([0.010, 0.014, 0.032])
    E.amb = BASE["amb"] * (1 - night) + na * night
    E.bounce = BASE["bounce"] * (1 - night) + na * 0.5 * night
    E.haze = BASE["haze"] * (1 - night) * (0.3 + 0.7 * k) + np.array([0.025, 0.022, 0.04]) * night
    E.fog = BASE["fog"]
    T.NIGHT[0] = smooth(0.02, -0.13, el)
    black = 0.0
    if f >= F_CHASE:                                      # the sky going to black while we orbit the airship
        d = cam_dist(min(f - F_CHASE, NCH))
        black = smooth(D_ORB0 + 40.0, D_ORB1, d)
    E.haze = E.haze * (1 - black) + VIOLET_HOR * 0.5 * black
    T.LOW_HAZE[0] = np.array([0.42, 0.26, 0.40]) * (1 - 0.7 * black)
    STATE.update(f=f, el=el, night=night, black=black)

def exposure_night(z):
    return 0.9 + 1.1 * STATE.get("night", 0.0)

ORIG_SKY = T.sky_dusk
VIOLET_TOP = np.array([0.012, 0.004, 0.026])              # the night sky over the party (the stadium design's too)
VIOLET_HOR = np.array([0.050, 0.018, 0.060])

def sky_night(d, eye, clouds=True):
    """The sunset sky as the sun goes (train.py's, the sun where it is now), sinking into twilight blue and then
    night: stars, the town's glow low down, the stadium's glow in the haze."""
    n = STATE.get("night", 0.0)
    el = d[..., 1]
    col = ORIG_SKY(d, eye, clouds)
    dim = 1 - 0.93 * n
    tw = np.array([0.020, 0.030, 0.085]) * (1 - np.clip(el / 0.6, 0, 1))[..., None] + np.array([0.004, 0.006, 0.02])
    col = col * dim + tw * n
    glow = n * 0.06 * np.exp(-np.maximum(el, 0) / 0.04)
    col = col + glow[..., None] * np.array([1.0, 0.55, 0.25])
    c = AUD_C + np.array([BX, 45.0, 0.0]) - eye
    c = c / np.linalg.norm(c)
    cosang = np.clip(d @ c, -1, 1)
    sg = n * (0.35 * np.exp(-(1 - cosang) / 0.004) + 0.08 * np.exp(-(1 - cosang) / 0.03))
    bk = STATE.get("black", 0.0)                          # deepening to dark violet (not black) while we orbit
    hz = np.exp(-np.maximum(el, 0) * 9)[..., None]
    col = col * (1 - bk) + (VIOLET_TOP * (1 - hz) + VIOLET_HOR * hz) * bk
    if n > 0.3:
        az = np.arctan2(d[..., 0], d[..., 2])
        cx, cy = np.floor(az * 900), np.floor(el * 900)
        h = T.hash2(cx, cy)
        star = (h > 0.9975) & (el > 0.06)
        col = col + (star * (h - 0.9975) * 400 * (n - 0.3))[..., None] * np.array([0.8, 0.85, 1.0])
    sg = sg * (1 + 1.5 * bk)                              # the bowl's glow over the rim stays, stronger in the dark
    return col + sg[..., None] * np.array([0.65, 0.45, 1.0])

# ---- the chase: one path, the camera and the drone on it, the camera a little behind ------------------------------

def catmull(pts, s):
    P = np.asarray(pts, float)
    n = len(P) - 1
    x = min(max(s, 0.0), 1.0) * n
    i = min(int(x), n - 1)
    u = x - i
    p0, p1, p2, p3 = P[max(i - 1, 0)], P[i], P[i + 1], P[min(i + 2, n)]
    return 0.5 * ((2 * p1) + (-p0 + p2) * u + (2 * p0 - 5 * p1 + 4 * p2 - p3) * u * u + (-p0 + 3 * p1 - 3 * p2 + p3) * u ** 3)

EYE_STOP = np.array([0.0, T.EYE_Y, ZS])
PUSH_END = np.array([0.15, 4.3, ZS + 5.0])
PANEL = dict(x=-3.5, y0=1.25 + 0.85, y1=1.25 + 1.6, z0=ZS + 8.53, z1=ZS + 9.53)   # the post's box, 1 x 0.75 m, facing +x
PANEL_C = np.array([PANEL["x"], (PANEL["y0"] + PANEL["y1"]) / 2, (PANEL["z0"] + PANEL["z1"]) / 2])
VIEW = np.array([-2.48, 2.47, ZS + 9.03])                 # where we stop to watch it, close: it fills the frame
SEG1_PTS = [VIEW, VIEW]
DRONE_HOVER = np.array([-6.6, 5.4, ZS + 12.2])            # it drops in under the canopy, up the platform
DRONE_FROM = np.array([-20.0, 22.0, ZS + 30.0])
T_H = 3.0                                                 # the hand-off: their camera at t = 3 s (36 frames, on a beat)

def _their_cam(t):
    """Their camera's eye and target at t (their frame), from their prototype; their keys if it isn't there."""
    sys.path.insert(0, AUDIENCE)
    try:
        import stadium as au
        e, g = au.camera(t)
        return np.asarray(e, float), np.asarray(g, float)
    except Exception:
        return np.array([-28.0, -140.0, 55.0]), np.array([0.0, 30.0, 8.0])

# the airship, south-west of the stadium and north-west of the station, high, its screen toward the station
BLIMP_AU = np.array([0.0, -10.0, 125.0])                  # over the middle of the bowl, high
BSC = 1.4                                                 # the model, scaled up: 154 m long
BLIMP_AU_V = 0.4                                          # metres a second along X
BLIMP_SCREEN_AU = BLIMP_AU + np.array([4.0 * BSC, -24.5 * BSC, 2.0 * BSC])
READ_AU = BLIMP_AU + np.array([-3.0, -112.0, -12.0])     # where we slow to read its screen, in front, a little below
ORBIT_C, ORBIT_R = BLIMP_AU[:2], np.array([104.0, 108.0])
ORBIT_TH = np.radians(np.arange(-95.0, -456.0, -30.0))    # clockwise from the front: once round
ORBIT = [np.array([ORBIT_C[0] + ORBIT_R[0] * math.cos(a), ORBIT_C[1] + ORBIT_R[1] * math.sin(a),
                   112.0 - 4.0 * math.sin(a)]) for a in ORBIT_TH]
# the wall: hugging the stadium's outer wall (an ellipse 134.6 x 114.6 m) 7 m out, 7 m up, east along its south side
WALL_R = np.array([134.6 + 17.0, 114.6 + 17.0])
def wall_pt(phi, z):
    return np.array([WALL_R[0] * math.cos(math.radians(phi)), WALL_R[1] * math.sin(math.radians(phi)), z])
WALL = [wall_pt(-130.0, 14.0), wall_pt(-122.0, 11.0), wall_pt(-114.0, 11.0), wall_pt(-106.0, 13.0), wall_pt(-99.0, 18.0)]
CREST_AU = np.array([-48.0, -110.0, 48.0])                # over the top of the south wall (36 m), the stage ahead
HANDOFF = to_film(CREST_AU)                               # the reveal: the hand-off to the stadium's sweep
HAND_LOOK = to_film(np.array([0.0, 40.0, 10.0]))          # the bowl, the crowd and the stage at once
AFTER = [np.array([-43.0, -96.0, 38.0]), np.array([-37.0, -82.0, 26.0]), np.array([-30.0, -68.0, 15.0]),
         np.array([-24.0, -54.0, 8.0]), np.array([-18.0, -42.0, 5.0])]
# the approach: off the platform, down to the street, straight at the stadium (two seconds); a left turn at its
# face, and up it (the wall is sheer, 36 m, the canopy's lip at 38: hugged 9 m off), round to the south; over the top
def face_pt(phi, z, off=9.0):
    return np.array([(134.6 + off) * math.cos(math.radians(phi)), (114.6 + off) * math.sin(math.radians(phi)), z])
APPROACH = [np.array([225.0, -9.5, 2.6]), np.array([190.0, -11.0, 2.4]), np.array([160.0, -13.0, 3.0]),
            face_pt(-12.0, 4.0), face_pt(-26.0, 10.0), face_pt(-40.0, 18.0), face_pt(-53.0, 27.0),
            face_pt(-63.0, 36.0), face_pt(-70.0, 45.0, 2.0), face_pt(-76.0, 52.0, -14.0), np.array([22.0, -96.0, 66.0]),
            np.array([8.0, -110.0, 88.0])]
HAND_V = 2.2                                              # metres a frame at the crest: slow, for the reveal
T_H = 3.0                                                 # their show clock at the reveal (on a beat)
PATH = [tuple(VIEW), (-4.6, 4.2, ZS + 10.4), (-8.6, 4.6, ZS + 11.0), (-14.0, 3.0, ZS + 11.0)] + \
       [tuple(to_film(q)) for q in APPROACH] + [tuple(to_film(READ_AU))] + \
       [tuple(to_film(q)) for q in ORBIT] + \
       [tuple(to_film(np.array(q))) for q in ((-35.0, -132.0, 92.0), (-52.0, -128.0, 66.0))] + \
       [tuple(HANDOFF)] + [tuple(to_film(q)) for q in AFTER]
FOLLOW = 7.5                                              # the drone leads by this much

def _arc(P):
    return np.concatenate([[0.0], np.cumsum(np.linalg.norm(np.diff(P, axis=0), axis=1))])

def _resample(P, step):
    L = _arc(P)
    s = np.arange(0.0, L[-1], step)
    return np.stack([np.interp(s, L, P[:, k]) for k in range(3)], 1)

def _smooth_path():
    """The flight path: the Catmull-Rom curve through PATH, then smoothed along its length (a 12 m Gaussian, so its
    curvature runs on without kinks), held exact at its start (the cab) and over the last 80 m (their camera)."""
    raw = np.array([catmull(PATH, x) for x in np.linspace(0, 1, 8001)])
    Q = _resample(raw, 1.0)
    L = _arc(Q)
    hand = L[np.argmin(np.linalg.norm(Q - HANDOFF, axis=1))]
    taper = np.array([smooth(2.0, 30.0, l) for l in L])[:, None]
    for _ in range(300):                                  # sigma ~ sqrt(300 / 2) ~ 12 samples of 1 m
        avg = Q.copy()
        avg[1:-1] = 0.25 * Q[:-2] + 0.5 * Q[1:-1] + 0.25 * Q[2:]
        Q = Q + taper * (avg - Q)
    return _resample(Q, 0.25)

_P = _smooth_path()
_L = _arc(_P)

def along(d):
    d = min(max(d, 0.0), _L[-1])
    i = min(max(int(np.searchsorted(_L, d)), 1), len(_L) - 1)
    u = (d - _L[i - 1]) / max(_L[i] - _L[i - 1], 1e-9)
    return _P[i - 1] + (_P[i] - _P[i - 1]) * u

def _nearest(p, after=0.0):
    dist = np.linalg.norm(_P - p, axis=1) + (_L < after) * 1e9
    return float(_L[np.argmin(dist)])

D_HAND = _nearest(HANDOFF)
D_READ = _nearest(to_film(READ_AU))
D_ORB0 = _nearest(to_film(ORBIT[0]))
D_ORB1 = _nearest(to_film(ORBIT[-1]), D_ORB0 + 200.0)
D_WALL0 = D_WALL1 = D_ORB1                                # (no wall skim now: straight from the orbit into the bowl)
D_TURN = _nearest(to_film(APPROACH[2]))                  # the end of the street run: the turn under the lip
VSTREET = 5.2                                             # the street run: 62 m/s, flat out
VMAX, VREAD, VORB, VWALL, ACC, DEC = 3.5, 1.1, 6.8, 3.2, 0.14, 0.05   # metres a frame; ACC, DEC: the most a frame

def _speed(d):
    """The speed wanted at d along the path: a climb out of the station, slow in front of the airship's screen,
    steady round it, their camera's speed at the hand-off."""
    if d < D_TURN - 10:
        return VSTREET
    if d < D_READ - 30:
        return VMAX
    if d < D_READ + 12:
        return VREAD
    if d < D_ORB1:
        return VORB
    if d < D_WALL1 + 10:
        return VWALL
    return HAND_V

def _profile():
    """Frame by frame: the speed eased toward the wanted one (looking ahead to brake in time), never changing by
    more than ACC a frame, then smoothed; the distances scaled to end exactly at the hand-off."""
    d, v, D = 0.0, 0.0, [0.0]
    while d < D_HAND:
        dec = DEC * (3.0 if d < D_TURN + 20 else 1.0)     # into the turn under the lip: braking hard
        look_ = v * v / (2 * dec) + 10.0
        want = min(_speed(d + x) for x in np.linspace(0.0, look_, 24))
        acc = 0.42 if d < D_TURN else ACC                 # the launch off the platform: hard
        v = min(max(want, v - dec), v + acc * min(1.0, 0.25 + len(D) / 8.0))
        v = max(v, 0.05)
        d += v
        D.append(d)
    for _ in range(APPEND + 2):                            # on past the crest at the reveal's speed
        D.append(D[-1] + HAND_V)
    D = np.array(D)
    V = np.diff(D)
    k = np.exp(-0.5 * (np.arange(-12, 13) / 4.0) ** 2); k /= k.sum()
    Vs = np.convolve(np.pad(V, 12, mode="edge"), k, mode="valid")
    D = np.concatenate([[0.0], np.cumsum(Vs)])
    n = len(D) - APPEND - 3
    D = D * (D_HAND / D[n])
    return D, n

_D, NCH = _profile()
F_HAND = F_CHASE + NCH                                    # the hand-off frame: their camera(T_H)
F_READ = F_CHASE + int(np.searchsorted(_D, D_READ))       # in front of the airship's screen, slowest
VOL_ON = F_READ - 46                                      # the screen turns to the volume as we come up to it: at 10,
VOL_11A, VOL_11B = F_READ - 30, F_READ - 20               # the 11 materialising,
VOL_CLICK = F_READ - 12                                   # the click to 11 (VOL 11 in red),
VOL_OFF = F_READ + 20                                     # held, then DEMOSPLASH 0x10
# The film skips the orbit round the airship: it shows the design's frames up to the click to
# VOL 11 (VOL_OFF) as they are, then CUT_N frames on (7 bars at 137.14 BPM: the beat runs on), straight into the
# dive; the camera moves from the read onto the dive over JOIN frames (the eye along a cubic from the read's place and
# speed to the dive's, the way, the bank and the lens eased between). Times below are the design's; the film's frame
# k is the design's design_f(k).
CUT_N, JOIN = 147, 24
F_FILM = F_HAND - CUT_N                                   # the film's hand-off frame (373)
NFR = F_HAND + 1 + APPEND                                 # (the design's frames)
NFR_FILM = NFR - CUT_N

def design_f(k):
    return k if k <= VOL_OFF else k + CUT_N

def cam_dist(fc):
    return float(np.interp(fc, np.arange(len(_D)), _D))

def cam_speed(fc):
    return cam_dist(fc + 0.5) - cam_dist(fc - 0.5)

def drone_pos(f):
    """The drone, in the film's unbent frame: hovering just ahead of the nose, then leading the camera along the
    path; aside while we read the screen; away up over the stadium as we orbit."""
    bob = np.array([0.05 * math.sin(f * 0.9), 0.08 * math.sin(f * 0.6), 0.0])
    fc = f - F_CHASE
    dd = cam_dist(max(fc, 0.0))
    lead = along(dd + FOLLOW) + np.array([0.6, -0.9, 0.0])
    a = f - (F_BTN + 1)                                   # it arrives, swooping down from ahead
    arr = DRONE_FROM + (DRONE_HOVER - DRONE_FROM) * ease(a / 13.0) + np.array([0.0, 1.5, 0.0]) * math.sin(math.pi * min(max(a / 13.0, 0), 1))
    w = ease((f - F_CHASE) / 18.0)
    p = (arr + bob) * (1 - w) + lead * w
    side = math.exp(-((dd - D_READ) / 60.0) ** 2)
    p = p + np.array([-14.0, -6.0, 0.0]) * side
    u = ease((dd - D_READ - 40.0) / 140.0) * (1 - ease((dd - D_ORB1 + 30.0) / 110.0))   # away while we orbit
    return p * (1 - u) + to_film(np.array([-150.0, -150.0, 190.0])) * u

def drone_dir(f):
    a, b = drone_pos(f - 0.5), drone_pos(f + 0.5)
    v = b - a
    return v if np.linalg.norm(v) > 1e-4 else np.array([0.0, 0.0, 1e-3])

def look(eye, tgt, roll=0.0):
    f = tgt - eye; f = f / np.linalg.norm(f)
    r = np.cross([0, 1.0, 0], f); r /= np.linalg.norm(r)
    u = np.cross(f, r)
    if roll:
        c, s_ = math.cos(roll), math.sin(roll)
        r, u = c * r + s_ * u, -s_ * r + c * u
    return np.array([r, u, f])

AU_HFOV = 2 * math.atan(768 / 1000.0)                     # their lens: focal 1000 at 1536 wide (75 degrees)

def cab_eye(t):
    tt = min(t, F_STOP)
    z = za(tt)
    v = speed(tt) / T.SPEED
    k = tt + T.NFRAMES
    sway = (0.035 * math.sin(k * 0.7) + 0.012 * math.sin(k * 2.3)) * v
    return np.array([sway, T.EYE_Y + 0.012 * math.sin(k * 1.9) * v, z])

_S1 = np.array([catmull(SEG1_PTS, x) for x in np.linspace(0, 1, 801)])
_S1L = np.concatenate([[0.0], np.cumsum(np.linalg.norm(np.diff(_S1, axis=0), axis=1))])

def seg1_eye(t):
    """At the post (after the dissolve): still but for a slow drift in, 15 cm over the shot."""
    u = min(max((t - F_GO) / (F_CHASE - F_GO), 0.0), 1.0)
    return VIEW + np.array([-0.15, 0.0, 0.0]) * (u - 1.0)

def chase_eye(t):
    if t <= F_CHASE:
        return seg1_eye(t)
    return along(cam_dist(t - F_CHASE))

def raw_aim(t):
    """Where the camera wants to look at frame t (a direction): ahead along the path, leading more the faster we
    go; the drone early on; the airship's screen while we read it; the airship while we orbit it; their aim at the
    hand-off."""
    if t <= F_CHASE:                                      # up the platform, the drone swooping in
        e = seg1_eye(t)
        a = T.nrm(np.array([-1.2, 0.6, 6.0]))
        wd = ease((t - F_BTN) / 12.0)
        return T.nrm(a * (1 - wd) + T.nrm(drone_pos(t) - e) * wd)
    if t >= F_HAND:
        return T.nrm(HAND_LOOK - chase_eye(t))
    fc = t - F_CHASE
    d, v = cam_dist(fc), cam_speed(fc)
    eye = chase_eye(t)
    A = along(d + 18.0 + 5.0 * v) - np.array([0.0, 2.0, 0.0])
    wdr = ease((t - F_PUSH) / 10.0) * (1 - ease((d - D_READ + 80.0) / 90.0))
    A = A * (1 - 0.5 * wdr) + drone_pos(t) * 0.5 * wdr
    ws = math.exp(-((d - D_READ) / (80.0 if d < D_READ else 45.0)) ** 2)
    A = A * (1 - ws) + (to_film(blimp_au(t)) + (to_film(BLIMP_SCREEN_AU) - to_film(BLIMP_AU))) * ws
    wo = min(ease((d - D_READ - 20.0) / 90.0), 1 - ease((d - D_ORB1 + 120.0) / 170.0))
    A = A * (1 - wo) + (to_film(blimp_au(t)) - np.array([0.0, 6.0, 0.0])) * wo
    dirn = T.nrm(A - eye)
    w = max(ease((fc - (NCH - 30)) / 30.0), 0.8 * ease((d - D_ORB1 + 30.0) / 50.0))   # the dive: on the bowl
    hd = T.nrm(HAND_LOOK - eye) * (1 - ease((fc - (NCH - 30)) / 30.0)) + T.nrm(HAND_LOOK - HANDOFF) * ease((fc - (NCH - 30)) / 30.0)
    return T.nrm(dirn * (1 - w) + hd * w)

def raw_roll(t):
    """Banking into the turns: from the path's turn rate and the speed (a coordinated turn, halved), level at the
    hand-off and in the cab."""
    if t <= F_CHASE + 1 or t >= F_HAND:
        return 0.0
    fc = t - F_CHASE
    d, v = cam_dist(fc), cam_speed(fc)
    a, b = along(d - 6.0), along(d + 6.0)
    c = along(d)
    h1, h2 = math.atan2(c[0] - a[0], c[2] - a[2]), math.atan2(b[0] - c[0], b[2] - c[2])
    dh = (h2 - h1 + math.pi) % (2 * math.pi) - math.pi
    kappa = dh / 6.0
    vs = v * T.FPS
    roll = -0.5 * math.atan(vs * vs * kappa / 9.81)
    return max(-0.35, min(0.35, roll)) * (1 - ease((fc - (NCH - 30)) / 30.0))

_AIM = {}

def aim(t):
    """The camera's aim and roll at t: the wanted ones averaged over a second (a Gaussian, 4 frames), narrowing to
    none at the hand-off so it lands on their aim exactly."""
    key = round(t, 3)
    if key not in _AIM:
        sig = 4.0 * min(1.0, max(0.0, (F_HAND - t) / 20.0))
        if sig < 0.3:
            _AIM[key] = (raw_aim(t), raw_roll(t))
        else:
            ks = np.arange(-12, 13)
            w = np.exp(-0.5 * (ks / sig) ** 2)
            dirs = np.array([raw_aim(t + k) for k in ks])
            ks2 = np.arange(-24, 25)
            w2 = np.exp(-0.5 * (ks2 / (2.5 * sig)) ** 2)
            rolls = np.array([raw_roll(t + k) for k in ks2])
            _AIM[key] = (T.nrm((dirs * w[:, None]).sum(0)), float((rolls * w2).sum() / w2.sum()))
    return _AIM[key]

FORCE_CAB = [False]

def _cam_parts(t):
    """The flight's eye (unbent), way, bank and field of view at the design's time t."""
    dirn, roll = aim(t)
    return chase_eye(t), dirn, roll, T.HFOV + (AU_HFOV - T.HFOV) * ease(max(0.0, t - F_CHASE) / NCH)

class NightCam:
    """The camera at frame t of the extension (the design's time)."""
    def __init__(self, t, W, H):
        self.W, self.H = W, H
        hfov = T.HFOV
        off = np.array([BX, 0.0, 0.0])
        j0 = VOL_OFF + CUT_N
        if t < F_GO or (FORCE_CAB[0] and t < F_VIEW):     # the cab (and the cab's half of the dissolve)
            eye = cab_eye(t) + off
            basis = look(eye, eye + np.array([0.0, -0.05, 1.0]))
        elif j0 < t < j0 + JOIN:                          # the join: from the read onto the dive
            a, a1 = _cam_parts(VOL_OFF), _cam_parts(VOL_OFF - 1)
            b, b1 = _cam_parts(j0 + JOIN), _cam_parts(j0 + JOIN - 1)
            bt = _cam_parts(t)
            u = (t - j0) / JOIN
            h10, h01, h11 = u ** 3 - 2 * u ** 2 + u, -2 * u ** 3 + 3 * u ** 2, u ** 3 - u ** 2
            eye = a[0] + h10 * JOIN * (a[0] - a1[0]) + h01 * (b[0] - a[0]) + h11 * JOIN * (b[0] - b1[0]) + off
            w = ease(u)
            dirn = T.nrm(a[1] * (1 - w) + bt[1] * w)
            basis = look(eye, eye + dirn * 50.0, a[2] * (1 - w) + bt[2] * w)
            hfov = a[3] * (1 - w) + bt[3] * w
        else:                                             # through the glass, to the panel, and the chase: one path
            eye0, dirn, roll, hfov = _cam_parts(t)
            eye = eye0 + off
            basis = look(eye, eye + dirn * 50.0, roll)
        self.f = (W / 2) / math.tan(hfov / 2)
        self.t, self.eye, self.basis, self.z = t, eye, basis, float(eye[2])

    def project(self, P):
        c = (P - self.eye) @ self.basis.T
        z = np.maximum(c[..., 2], 1e-3)
        return self.W / 2 + self.f * c[..., 0] / z, self.H / 2 - self.f * c[..., 1] / z, c[..., 2]

# ---- the blimp: drifting over the stadium's approach, its screen toward the station ------------------------------

# The airship hangs over the stadium's bowl, high, drifting slowly: in the audience design's frame (X right, Y to
# the stage, Z up; t their show clock, 0 at the hand-off) its centre is (BLIMP_AU[0] + BLIMP_AU_V * t, BLIMP_AU[1],
# BLIMP_AU[2] + 1.2 sin(0.6 t)), its hull along X, its screen facing -Y (toward the station and the cameras coming in).
BS = dict(c=np.array([4.0, 2.0, -24.5]), w=33.0, h=18.6, tilt=0.26)   # its screen (models/build_night_models.py)
BS["n"] = np.array([0.0, -math.sin(BS["tilt"]), -math.cos(BS["tilt"])])
BS["up"] = np.array([0.0, math.cos(BS["tilt"]), -math.sin(BS["tilt"])])

def blimp_au(f):
    t = (f - F_HAND) / T.FPS + T_H
    return BLIMP_AU + np.array([BLIMP_AU_V * t, 0.0, 1.2 * math.sin(0.6 * t)])

def blimp_pos(f):
    """Its centre in the film's unbent frame."""
    return to_film(blimp_au(f))

# lights from the stadium aimed up at it: (their source position, the spot on the belly it holds, in the airship's
# own frame: x along, y up, z across)
UPLIGHTS = [((-120.0, -50.0, 37.5), 0.0), ((120.0, -50.0, 37.5), 1.3), ((-70.0, -105.0, 37.5), 2.1),
            ((70.0, -105.0, 37.5), 3.4), ((0.0, 40.0, 4.0), 4.0), ((-120.0, 40.0, 37.5), 5.2), ((120.0, 40.0, 37.5), 0.7),
            ((0.0, -112.0, 37.5), 2.8)]

def uplight_world(k, f):
    """The stadium's spots playing over the airship: each sweeps its spot along the hull and round its flanks
    (and across the screen), slowly, each at its own phase."""
    src, ph = UPLIGHTS[k]
    o = aud_world(np.array(src))
    t = f / T.FPS
    u = 0.85 * math.sin(0.55 * t + ph)                    # along the hull
    v = -math.pi / 2 + 1.25 * math.sin(0.4 * t + ph * 1.7)   # round it: from under the belly up the flanks
    tgt = blimp_hull(u, v) * 1.02
    p = blimp_pos(f) + np.array([BX, 0.0, 0.0]) + tgt * BSC
    return o, p

def blimp_hull(u, v):
    """The hull's surface, as the model builds it (u along -1..1, v round)."""
    L, RW, RH = 55.0, 21.0, 15.0
    base = max(0.0, 1 - abs(u) ** 2.2) ** 0.55
    side, up = math.cos(v), math.sin(v)
    lobe = (0.5 + 0.5 * math.cos(u * math.pi * 6.5)) ** 2 * max(0.0, up + 0.35) * abs(side) ** 0.5
    r = 1 + 0.16 * lobe - 0.10 * max(0.0, -up) ** 2
    return np.array([L * u, RH * base * r * up * (1.15 if up > 0 else 0.85), RW * base * r * side])

def _blimp_lights():
    rng = np.random.default_rng(2019)
    pts, kinds = [], []
    for k in range(520):                                  # running lights all over the hull
        u, v = rng.uniform(-0.92, 0.92), rng.uniform(0, 2 * np.pi)
        pts.append(blimp_hull(u, v) * 1.03); kinds.append(0)
    for u in np.linspace(-0.9, 0.9, 60):                  # a row along each flank's waist
        for v in (0.15, np.pi - 0.15, -0.6, np.pi + 0.6):
            pts.append(blimp_hull(u, v) * 1.03); kinds.append(0)
    w, h = BS["w"] + 4.5, BS["h"] + 4.5                    # a string of lights round the screen's frame
    for t in np.linspace(0, 1, 90, endpoint=False):
        per = 2 * (w + h); d = t * per
        if d < w: a, b = -w / 2 + d, h / 2
        elif d < w + h: a, b = w / 2, h / 2 - (d - w)
        elif d < 2 * w + h: a, b = w / 2 - (d - w - h), -h / 2
        else: a, b = -w / 2, -h / 2 + (d - 2 * w - h)
        pts.append(BS["c"] + np.array([a, 0, 0]) + BS["up"] * b + BS["n"] * 0.8); kinds.append(1)
    for k in range(24):                                   # red beacons on the masts' tips
        end = -1 if k % 2 else 1
        pts.append(blimp_hull(end * rng.uniform(0.8, 0.95), rng.uniform(0, 2 * np.pi)) * 1.0 +
                   np.array([end * rng.uniform(6, 20), rng.uniform(-8, 12), rng.uniform(-8, 8)])); kinds.append(2)
    return np.array(pts), np.array(kinds)

BLIMP_LIGHTS = _blimp_lights()

def blimp_text():
    """The screen's words as a mask over the whole screen (a font rasterised at prep: an asset)."""
    from PIL import Image, ImageDraw, ImageFont
    W_, H_ = 1000, 560
    im = Image.new("L", (W_, H_), 0)
    d = ImageDraw.Draw(im)
    f1 = ImageFont.truetype("/System/Library/Fonts/HelveticaNeue.ttc", 150, index=1)
    f2 = ImageFont.truetype("/System/Library/Fonts/HelveticaNeue.ttc", 170, index=1)
    for s, f, y in (("DEMOSPLASH", f1, 205), ("0x10", f2, 360)):
        w = d.textlength(s, font=f)
        d.text(((W_ - w) / 2, y), s, font=f, fill=255)
    return np.array(im, np.float32) / 255

BLIMP_TEXT = blimp_text()

_VT = {}

def vol_value(f):
    """The screen's knob: at 10, then one click to 11 (overshooting a hair the frame after)."""
    if f < VOL_CLICK:
        return 10.0
    return 10.6 if f < VOL_CLICK + 1 else (11.12 if f < VOL_CLICK + 2 else 11.0)

def vol_tex(f):
    """The airship's screen as a volume control, 1000 x 560: a huge knob, its scale 0-10 filling the whole sweep like
    any other (10 is the end stop), the pointer at 10; then a surprise 11 materialising past the end, in red,
    glowing, and the click to it; VOL 10 in white beside it, VOL 11 in red from the click."""
    from PIL import Image, ImageDraw, ImageFont
    kv = vol_value(f)
    m11 = smooth(VOL_11A, VOL_11B, f)
    red = f >= VOL_CLICK
    key = (kv, round(m11, 2), red)
    if key in _VT:
        return _VT[key]
    W_, H_ = 1000, 560
    im = Image.new("RGB", (W_, H_), (0, 0, 0))
    d = ImageDraw.Draw(im)
    for y in range(H_):                                   # a deep violet ground
        d.line([(0, y), (W_, y)], fill=(int(30 + 30 * y / H_), 6, int(48 + 40 * y / H_)))
    F = lambda n: ImageFont.truetype(CO.HEL, n, index=1)
    cx, cy, R = 290, 215, 130
    for v_ in range(12):
        a = math.radians(-135 + 27 * v_)                  # 0-10 over 270 degrees; the 11 past the stop
        if v_ == 11:
            if m11 <= 0:
                continue
            c = (255, int(40 + 215 * (1 - m11)), int(30 + 225 * (1 - m11)))   # out of white into red
            col = tuple(int(x * m11) for x in c)
        else:
            col = (235, 232, 225)
        x0, y0 = cx + math.sin(a) * R * 1.08, cy - math.cos(a) * R * 1.08
        x1, y1 = cx + math.sin(a) * R * (1.32 if v_ >= 10 else 1.22), cy - math.cos(a) * R * (1.32 if v_ >= 10 else 1.22)
        d.line([(x0, y0), (x1, y1)], fill=col, width=9 if v_ >= 10 else 5)
        fs = F(120 if v_ == 11 else (76 if v_ == 10 else 36))   # 10 and the surprise 11 big: read from the back
        rr = 1.8 if v_ == 11 else (1.7 if v_ == 10 else 1.5)
        tx, ty = cx + math.sin(a) * R * rr, cy - math.cos(a) * R * rr
        t = str(v_)
        d.text((tx - d.textlength(t, font=fs) / 2, ty - fs.size * 0.6), t, font=fs, fill=col)
    d.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(20, 20, 22), outline=(120, 120, 126), width=6)
    for k in range(40):                                   # the knurled edge
        a = 2 * math.pi * k / 40 + math.radians(27 * kv)
        d.line([(cx + math.sin(a) * R * 0.86, cy - math.cos(a) * R * 0.86), (cx + math.sin(a) * R, cy - math.cos(a) * R)],
               fill=(60, 60, 66), width=4)
    a = math.radians(-135 + 27 * kv)
    d.line([(cx + math.sin(a) * R * 0.2, cy - math.cos(a) * R * 0.2), (cx + math.sin(a) * R * 0.95, cy - math.cos(a) * R * 0.95)],
           fill=(255, 255, 250), width=16)
    t1, t2 = "VOL", ("11" if red else "10")
    col = (255, 36, 24) if red else (240, 238, 232)
    d.text((610, 120), t1, font=F(150), fill=col)
    d.text((600, 270), t2, font=F(200), fill=col)
    a_ = (np.array(im, np.float32) / 255) ** 2.2 * 2.2
    if m11 > 0:                                           # the 11 materialising: a red glow round it
        a11 = math.radians(162)
        gx, gy = cx + math.sin(a11) * R * 1.8, cy - math.cos(a11) * R * 1.8
        yy, xx = np.mgrid[0:H_, 0:W_]
        g = np.exp(-((xx - gx) ** 2 + (yy - gy) ** 2) / 6000.0) * (m11 * (1 - m11) * 4 + (0.6 if red else 0.2))
        a_ = a_ + g[..., None] * np.array([3.0, 0.3, 0.2])
    if VOL_CLICK <= f < VOL_CLICK + 2:                    # the click: the whole screen kicks
        a_ = a_ * 1.6
    _VT[key] = a_
    return a_

def vol_screen(a, b, f):
    tex = vol_tex(f)
    th, tw = tex.shape[:2]
    c = tex[np.clip(((1 - b) * th).astype(int), 0, th - 1), np.clip((a * tw).astype(int), 0, tw - 1)]
    scan = 0.85 + 0.15 * np.sin(b * 560 * np.pi)
    return c * scan[:, None]

def blimp_screen(a, b, f):
    """The airship's picture at screen coordinates a (0 left .. 1 right), b (0 bottom .. 1 top): a magenta glow
    over a horizon, an orange laser grid racing toward us under it, DEMOSPLASH 0x10 in blue over it all."""
    hz = 0.62
    col = np.zeros(a.shape + (3,))
    sky = np.clip((b - hz) / (1 - hz), 0, 1)
    glow = np.exp(-((b - 0.76) / 0.07) ** 2)
    col += (np.array([0.9, 0.08, 0.45]) * (1 - sky)[:, None] * 1.6 + np.array([1.4, 1.1, 0.5]) * glow[:, None] *
            np.exp(-((a - 0.55) / 0.3) ** 2)[:, None] * 2.0) * (b > hz)[:, None]
    below = b < hz
    Zg = 0.22 / np.maximum(hz - b, 1e-3)
    Xg = (a - 0.5) * Zg * 3.0
    fz = np.abs(((Zg * 1.2 - f * 0.12) % 1.0) - 0.5)
    fx = np.abs((Xg % 1.0) - 0.5)
    wz = 0.5 - 0.06 / np.maximum(Zg, 0.3) - 0.02
    wx = 0.5 - 0.035 - 0.0 * Zg
    line = np.maximum(T.smooth(wz, 0.5, fz), T.smooth(wx, 0.5, fx) * np.clip(1.5 / Zg, 0, 1))
    fade = np.clip((hz - b) / 0.08, 0, 1)
    col += (np.array([1.6, 0.75, 0.12]) * line[:, None] * fade[:, None] + np.array([0.12, 0.02, 0.08]) * fade[:, None]) * below[:, None]
    iy = np.clip(((1 - b) * (BLIMP_TEXT.shape[0] - 1)).astype(int), 0, BLIMP_TEXT.shape[0] - 1)
    ix = np.clip((a * (BLIMP_TEXT.shape[1] - 1)).astype(int), 0, BLIMP_TEXT.shape[1] - 1)
    tm = BLIMP_TEXT[iy, ix]
    sh = BLIMP_TEXT[np.clip(iy - 6, 0, None), np.clip(ix - 6, 0, None)]
    col = col * (1 - 0.85 * np.maximum(tm, sh)[:, None]) + np.array([0.3, 0.75, 2.4]) * tm[:, None] * 1.6
    scan = 0.85 + 0.15 * np.sin(b * 560 * np.pi)          # the picture's lines
    return col * scan[:, None]

# ---- the audience design's stadium ------------------------------------------------------------------------------

_AU = {}

def audience():
    """Their prototype as a module (its light paths and, for the dissolve, its renderer)."""
    if "mod" not in _AU:
        sys.path.insert(0, AUDIENCE)
        try:
            import stadium as au
            _AU["mod"] = au
        except Exception as e:                            # not there: block out, say so
            print("no audience stadium:", e, flush=True)
            _AU["mod"] = None
        _AU["json"] = json.load(open(AUWORK + "stadium/stadium.json")) if os.path.exists(AUWORK + "stadium/stadium.json") else None
    return _AU["mod"]

# ---- models and materials -----------------------------------------------------------------------------------------

K_PLATFORM, K_LED, K_NAV, K_DRONE, K_TUBE, K_SCREEN, K_TIER, K_PITCH, K_FENCE, K_STADWALL, K_EMITX, K_BHULL, \
    K_BSCREEN, K_BFRAME, K_DSCREEN, K_STADSKIN, K_PANEL = range(40, 57)
SCREENS = []                                              # the callouts' screens and signs (see build_screens)

BPM = 137.14
BEAT_F = T.FPS * 60.0 / BPM                               # frames a beat: 5.26

def beat(f):
    """1 on beats 1 and 3 of the soundtrack's 137 BPM (every other beat, on their show's clock), decaying."""
    tt = (f - F_HAND) / T.FPS + T_H
    ph = (tt * T.FPS / (2 * BEAT_F)) % 1.0
    return math.exp(-3.0 * ph)

def pulse(f, lo):
    """The main lights' level: from lo between beats to 1 on them, once it's dark."""
    n = STATE.get("night", 0.0)
    return 1.0 - n * (1 - lo) * (1 - beat(f))
EXTRA = {}

def own_roles(mats):
    roles = {r: mats.add(T.K_TRAIN, (0.70, 0.70, 0.69), (float(i), 1.0))
             for i, r in enumerate(["body", "stripe", "skirt", "winband", "screen", "lamp", "nose"])}
    roles["gangway"] = mats.add(T.K_DARK, (0.03, 0.03, 0.035))
    return roles

def stad_clear(x0, x1, z0, z1):
    c = AUD_C
    return not (x1 > c[0] - 142 and x0 < c[0] + 142 and z1 > c[2] - 122 and z0 < c[2] + 122)

def build_extra(g, mats):
    """The station, its board, the town, its street lights; the audience design's stadium; the own train and the
    drone (kept aside: they move)."""
    rng = np.random.default_rng(2026)
    M = T.model
    conc = mats.add(T.K_CONC, (0.45, 0.45, 0.44), (6.0,))
    steel = mats.add(T.K_METAL, (0.42, 0.43, 0.45))
    roles = {"platform": mats.add(K_PLATFORM, (0.48, 0.47, 0.45)), "fence": mats.add(K_FENCE, (0.62, 0.64, 0.66)),
             "steel": steel, "canopy": mats.add(T.K_CONC, (0.62, 0.62, 0.60), (0.0,)),
             "tube": mats.add(K_TUBE, (1.0, 0.97, 0.9)),
             "signsmall": mats.add(T.K_SIGN, (0.1, 0.3, 0.8), (17.0,)),
             "buildwall": mats.add(T.K_BUILD, (0.55, 0.56, 0.58), (2.0, 3.0, 0.0, 0.3, 16.0, T.LAND)),
             "trim": conc, "unit": mats.add(T.K_ROOFTOP, (0.6, 0.6, 0.6)),
             "namesign": mats.add(T.K_SIGN, (0.95, 0.95, 0.95), (3.0, 1.0))}
    g.place(M("station"), (0.0, 0.0, PL0), 0.0, roles=roles)
    led = mats.add(K_LED, (0.02, 0.02, 0.02))
    g.place(M("led_board"), (-4.6, 4.25, BOARD_Z), 0.0,
            roles={"frame": mats.add(T.K_DARK, (0.03, 0.03, 0.035)), "led": led, "steel": steel})
    EXTRA["board"] = dict(cx=-4.6, y0=4.25, z=BOARD_Z, w=9.6, h=2.9)
    nocull = [led]
    # the town
    siding = [(0.74, 0.71, 0.64), (0.66, 0.62, 0.55), (0.78, 0.77, 0.74), (0.58, 0.55, 0.50), (0.70, 0.62, 0.50)]
    tiles = [(0.10, 0.11, 0.14), (0.16, 0.13, 0.12), (0.20, 0.20, 0.21)]
    lamps = []
    acu = mats.add(T.K_ROOFTOP, (0.70, 0.70, 0.68))
    def clear(x0, x1, z0, z1):
        if z1 > PL0 - 40 and z0 < PL1 + 30 and x1 > -14 and x0 < X2 + 50:
            return False
        if z1 > ZS - 10 and z0 < ZS + 32 and x0 < -14:   # the avenue from the station to the stadium
            return False
        return stad_clear(x0, x1, z0, z1)
    for side in (-1, 1):
        x = 16.0
        while x < 150:
            for k in range(2):
                dep = float(rng.uniform(10, 13))
                z = TOWN0 + float(rng.uniform(0, 8))
                while z < PL1 + 600:
                    fr = float(rng.uniform(8, 12))
                    xa, xb = (MID - x - dep, MID - x) if side < 0 else (MID + x, MID + x + dep)
                    if clear(xa, xb, z, z + fr) and rng.random() > 0.08:
                        if rng.random() < 0.78:
                            nm = "house_%d" % int(rng.integers(8))
                            wm = mats.add(T.K_CHOUSE, siding[int(rng.integers(len(siding)))],
                                          (float(rng.uniform(0, 100)), T.LAND + 6.0, T.LAND, float(rng.uniform(2.6, 3.8)),
                                           float(rng.integers(0, 3))))
                            r = {"wall": wm, "roof": mats.add(T.K_TILE, tiles[int(rng.integers(len(tiles)))]),
                                 "tin": mats.add(T.K_SHED, (0.15, 0.30, 0.55)), "trim": conc, "rail": conc,
                                 "unit": acu, "solar": mats.add(T.K_SOLAR, (0.03, 0.05, 0.12))}
                        else:
                            nm = "apartment_%d" % int(rng.integers(4))
                            wm = mats.add(T.K_BUILD, (0.60, 0.58, 0.54), (3.0, 3.0, 0.0, float(rng.uniform(0, 1)),
                                                                          T.LAND + 15.0, T.LAND))
                            r = {"wall": wm, "trim": conc, "rail": conc, "unit": acu, "tank": steel}
                        g.place(M(nm), ((xa + xb) / 2, T.LAND, z + fr / 2), float(rng.integers(4)) * math.pi / 2,
                                (0.95, 1.0, 0.95), r)
                    z += fr + 1.0
                x += dep + 1.0
            xs = MID + side * (x + 3.0)
            for zl in np.arange(TOWN0 + 10, PL1 + 600, 32.0):
                if clear(xs - 2, xs + 2, zl - 2, zl + 2):
                    g.place(M("street_light"), (xs, T.LAND, zl), 0.0 if side < 0 else math.pi,
                            roles={"steel": steel, "lamp": mats.add(K_NAV, (1.0, 0.85, 0.6), (3.0,))})
                    lamps.append((xs + (1.7 if side < 0 else -1.7), T.LAND + 7.35, zl))
            x += 6.0
    for zl in np.arange(TOWN0, PL0 - 20, 30.0):
        for xs in (MID - 9.0, MID + 9.0):
            g.place(M("street_light"), (xs, T.LAND, zl), 0.0 if xs < MID else math.pi,
                    roles={"steel": steel, "lamp": mats.add(K_NAV, (1.0, 0.9, 0.75), (3.0,))})
            lamps.append((xs + (1.7 if xs < MID else -1.7), T.LAND + 7.35, zl))
    EXTRA["lamps"] = np.array(lamps)
    build_screens(g, mats, steel)
    # the audience design's stadium: their geometry, in their axes, set down here (not their ground)
    audience()
    path = AUWORK + "stadium/stadium.npz"
    if _AU.get("json") is not None and os.path.exists(path):
        d = np.load(path)
        p, n, m = d["p"].astype(float), d["n"].astype(float), d["m"]
        keep = m != 0
        V = to_film(p[keep])
        Nn = np.stack([n[keep][..., 0], n[keep][..., 2], n[keep][..., 1]], -1)
        ids = []
        for mm in _AU["json"]["materials"]:
            if mm["name"] == "facade":
                ids.append(mats.add(K_STADSKIN, tuple(mm["rgb"])))
            elif mm["kind"] == "emit":
                ids.append(mats.add(K_EMITX, tuple(mm["rgb"]), (float(mm.get("emit", 1.0)),)))
            else:
                ids.append(mats.add(K_FENCE, tuple(mm["rgb"])))
        ids = np.array(ids)
        g.chunks.append((V, Nn, ids[m[keep]], 0, float(AUD_C[2])))
        nocull += list(ids)
        EXTRA["stadium"] = "audience"
    else:                                                 # their stadium isn't there: the block-out
        sroles = {"tier": mats.add(K_TIER, (0.05, 0.05, 0.06)), "outer": mats.add(K_STADWALL, (0.16, 0.16, 0.18)),
                  "flood": mats.add(K_TUBE, (1.0, 1.0, 0.95)), "stage": mats.add(T.K_DARK, (0.04, 0.04, 0.05)),
                  "screen": mats.add(K_SCREEN, (0.0, 0.0, 0.0)), "pitch": mats.add(K_PITCH, (0.02, 0.02, 0.025))}
        g.place(M("stadium"), tuple(AUD_C), 0.0, roles=sroles)
        nocull += [sroles["tier"], sroles["outer"], sroles["screen"]]
        EXTRA["stadium"] = "block-out"
    EXTRA["nocull"] = nocull
    # our own train, nose toward +z, and the drone
    og = T.Geo()
    ro = own_roles(mats)
    for c in range(T.CARS):
        z0 = -c * T.CAR
        if c == 0:
            og.place(M("n700_nose"), (0.0, 0.0, 0.0), math.pi, roles=ro)
        elif c == T.CARS - 1:
            og.place(M("n700_nose"), (0.0, 0.0, z0 - T.CAR), 0.0, roles=ro)
        else:
            og.place(M("n700_mid"), (0.0, 0.0, z0), math.pi, roles=ro)
        if c in (4, 12):
            og.place(M("pantograph"), (0.0, 0.0, z0 - 12.0), 0.0, roles={"gangway": ro["gangway"]})
    EXTRA["own"] = og.arrays()
    dg = T.Geo()
    dg.place(M("drone"), (0, 0, 0), 0.0, (1.3, 1.3, 1.3), roles={
        "dronebody": mats.add(K_DRONE, (0.10, 0.10, 0.11)), "rotor": mats.add(T.K_DARK, (0.05, 0.05, 0.05)),
        "navred": mats.add(K_NAV, (1.0, 0.05, 0.03), (0.0,)), "navgreen": mats.add(K_NAV, (0.05, 1.0, 0.2), (1.0,)),
        "strobe": mats.add(K_NAV, (1.0, 1.0, 1.0), (2.0,))})
    EXTRA["drone"] = dg.arrays()
    bg = T.Geo()
    bg.place(M("blimp"), (0, 0, 0), 0.0, roles={
        "hull": mats.add(K_BHULL, (0.055, 0.055, 0.065)), "greeble": mats.add(K_BHULL, (0.085, 0.08, 0.085)),
        "mast": mats.add(T.K_METAL, (0.12, 0.12, 0.13)), "bscreen": mats.add(K_BSCREEN, (0.0, 0.0, 0.0)),
        "bframe": mats.add(K_BFRAME, (1.0, 0.28, 0.06))})
    EXTRA["blimp"] = bg.arrays()
    EXTRA["nocull"] += [int(m) for m in np.unique(EXTRA["blimp"][2])]

def add_screen(g, mats, c, n, w, h, demo, kind, steel, pole=True, box=True):
    """A screen (kind clip: the demo's clip with its caption; banner: its title and group) centred at c (unbent)
    facing n, w x h metres, in a dark frame, on a pole down to the ground."""
    n = np.asarray(n, float); n = n / np.linalg.norm(n)
    r = np.cross([0, 1.0, 0], n); r = r / np.linalg.norm(r)
    u = np.cross(n, r)
    c = np.asarray(c, float)
    idx = len(SCREENS)
    m = mats.add(K_DSCREEN, (0.0, 0.0, 0.0), (float(idx),))
    SCREENS.append(dict(c=c, r=-r, u=u, w=w, h=h, demo=demo, kind=kind))
    q = lambda a, b, o=0.0: tuple(c - r * a + u * b + n * o)
    g.quad(q(-w / 2, -h / 2, 0.05), q(w / 2, -h / 2, 0.05), q(w / 2, h / 2, 0.05), q(-w / 2, h / 2, 0.05), tuple(n), m)
    if box:
        dk = mats.add(T.K_DARK, (0.03, 0.03, 0.035))
        lo = np.minimum.reduce([np.array(q(sa * (w / 2 + 0.4), sb * (h / 2 + 0.4), so)) for sa in (-1, 1) for sb in (-1, 1) for so in (0.0, -0.8)])
        hi = np.maximum.reduce([np.array(q(sa * (w / 2 + 0.4), sb * (h / 2 + 0.4), so)) for sa in (-1, 1) for sb in (-1, 1) for so in (0.0, -0.8)])
        if abs(n[0]) < 0.2 or abs(n[2]) < 0.2:            # an axis-aligned frame only where it fits
            g.box(tuple(lo - n * 0.0), tuple(hi), dk)
    if pole:
        base = c - n * 0.6
        g.member((base[0], T.LAND, base[2]), (base[0], c[1] - h / 2, base[2]), max(0.4, w / 30), max(0.4, w / 30), steel)

def build_screens(g, mats, steel):
    """The callouts along the way: billboards beside the line (the cab sees them coming), boards hung under the
    platforms' canopies, screens on towers along the flight."""
    rng = np.random.default_rng(1010)
    SCREENS.clear()
    n_ = len(CO.ENTRIES)
    k = 0
    for i, z in enumerate(np.arange(Z0 + 120.0, PL0 + 150.0, 36.0)):      # billboards beside the line
        side = 1 if i % 2 else -1
        x = MID + side * float(rng.uniform(14, 20))
        y = float(rng.uniform(10, 14))
        if PL0 - 10 < z < PL1 + 10:
            y += 2.0
        add_screen(g, mats, (x, y, z), (-0.35 * side, 0.0, -1.0), 12.0, 9.0, k % n_, "clip", steel)
        k += 1
    for x, z in ((-6.1, ZS - 28.0), (X2 + 6.1, ZS - 12.0), (X2 + 6.1, ZS + 22.0), (-6.1, ZS + 34.0)):  # platform boards
        add_screen(g, mats, (x, 6.05, z), (0.0, 0.0, -1.0), 3.8, 1.5, k % n_, "banner", steel, pole=False)
        k += 1
    for x, z, h in ((-45.0, ZS - 260.0, 62.0), (60.0, ZS - 215.0, 78.0), (-52.0, ZS - 160.0, 70.0),
                    (75.0, ZS - 120.0, 84.0)):   # beside the line before the station: the cab sees them coming   # screen towers along the flight
        wm = mats.add(K_FENCE, (0.06, 0.06, 0.07))         # a dark tower: its screens are the point
        g.box((x - 11, T.LAND, z), (x + 11, T.LAND + h, z + 16), wm)
        nrm_ = np.array([-0.25 * np.sign(x), 0.0, -1.0])
        add_screen(g, mats, (x, T.LAND + h - 13.0, z - 0.6), nrm_, 24.0, 18.0, k % n_, "clip", steel, pole=False, box=False)
        k += 1
        add_screen(g, mats, (x, T.LAND + h - 32.0, z - 0.6), nrm_, 20.0, 6.0, k % n_, "banner", steel, pole=False, box=False)
        k += 1
    EXTRA["screens_used"] = k

ORIG_SCENE = T.scene
ORIG_VISIBLE = T.visible

def scene_night():
    S = ORIG_SCENE()
    if "night" not in S:
        V, KIND = S["V"], S["mats"][0][S["M"]]
        c = V.mean(1)
        x, z = c[:, 0] - BX, c[:, 2]
        ext = np.arange(len(V)) >= EXTRA["first_new_chunk"]
        farm = np.isin(KIND, (T.K_TREE, T.K_WOOD, T.K_HOUSE, T.K_TILE, T.K_SHED, T.K_SOLAR))
        posts = (KIND == T.K_CONC) & (np.abs(np.abs(x - MID) - 7.6) < 0.3)
        in_stad = (((x - AUD_C[0]) / 142.0) ** 2 + ((z - AUD_C[2]) / 122.0) ** 2 < 1)   # inside its wall
        y = c[:, 1]
        by_panel = ((KIND == K_FENCE) | (KIND == T.K_METAL)) & (np.abs(x + 2.3) < 0.3) & (y < 3.0) & \
            (np.abs(z - PANEL_C[2]) < 14.0)
        S["by_panel"] = by_panel
        drop = ~ext & ((farm & (z > TOWN0)) | (posts & (z > TOWN0 - 40)) | in_stad |
                       (farm & (z > PL0 - 60) & (z < PL1 + 80) & (np.abs(x - MID) < 70)))
        town = ext & ~(((z > PL0 - 60) & (z < PL1 + 80) & (np.abs(x - MID) < 60)) | in_stad)
        S["small"] = S["small"] | town
        keep = ~drop
        for k_ in ("V", "N", "M", "zlo", "zhi", "small", "xmid", "lod", "iz"):
            S[k_] = S[k_][keep]
        for m in EXTRA["nocull"]:
            T.NOCULL[m] = True
        T.POLES[:] = [zp for zp in T.POLES if zp < TOWN0 - 20]
        S["night"] = True
    return S

def build_hook(g, mats):
    EXTRA["first_new_chunk"] = len(g.M) + sum(len(c[2]) for c in g.chunks)
    build_extra(g, mats)

def visible_night(S, cam):
    zlo, zhi, small = S["zlo"], S["zhi"], S["small"]
    sel = (zhi > cam.z - 1400) & (zlo < cam.z + 1600)
    sel &= ~small | ((zlo < cam.z + 650) & (zhi > cam.z - 650))
    near = np.abs(S["iz"] - cam.z) < 260
    sel &= (S["lod"] == 0) | ((S["lod"] == 1) & near) | ((S["lod"] == 2) & ~near)
    return sel

# ---- what moves -------------------------------------------------------------------------------------------------

def place_arrays(arr, pos, yaw=0.0, pitch=0.0):
    V, N, M = arr
    cy, sy = math.cos(yaw), math.sin(yaw)
    cp, sp = math.cos(pitch), math.sin(pitch)
    R = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]]) @ np.array([[1, 0, 0], [0, cp, -sp], [0, sp, cp]])
    return V @ R.T + pos, N @ R.T, M

def drone_pose(f):
    v = drone_dir(f)
    yaw = math.atan2(v[0], v[2])
    pitch = -0.3 * min(1.0, np.linalg.norm(v) / 2.0)        # nose down as it speeds up
    return drone_pos(f) + np.array([BX, 0.0, 0.0]), yaw, pitch

def dynamic(t, cam, S, TV):
    parts = [] if TV is None else [TV]
    f = STATE["f"]
    crossed = cam.eye[2] > ZS + 1.6                          # past the windscreen: our own train is outside now
    if f >= F_GO and crossed:
        T.OWN_FRONT[0] = TIP
        V, N, M = EXTRA["own"]
        parts.append((V + np.array([BX, 0.0, TIP]), N, M))
    if f >= F_BTN - 2:
        p, yaw, pitch = drone_pose(f)
        parts.append(place_arrays(EXTRA["drone"], p, yaw, pitch))
    V, N, M = EXTRA["blimp"]
    parts.append((V * BSC + blimp_pos(f) + np.array([BX, 0.0, 0.0]), N, M))
    if not parts:
        return None
    return (np.concatenate([p_[0] for p_ in parts]), np.concatenate([p_[1] for p_ in parts]),
            np.concatenate([p_[2] for p_ in parts]))

def vel_hook(Pa, Pb, kind, t):
    f = STATE["f"]
    dr = (kind == K_DRONE) | (kind == K_NAV)
    if f >= F_STOP - 30 and dr.any():
        sh = T.SHUTTER / 2
        p0 = drone_pos(f)
        Pa[dr] += drone_pos(f - sh) - p0; Pb[dr] += drone_pos(f + sh) - p0

# ---- materials ----------------------------------------------------------------------------------------------------

def led_bitmap():
    """The board's dots: the IBM VGA 8x16 font, a dot a font pixel; TONIGHT: over DEMOSPLASH 0x10, 128 x 40."""
    from PIL import Image, ImageDraw, ImageFont
    f = ImageFont.truetype(os.path.join(os.path.dirname(os.path.abspath(__file__)), "pcfonts", "Px437_IBM_VGA_8x16.ttf"), 16)
    im = Image.new("L", (128, 40), 0)
    d = ImageDraw.Draw(im)
    for i, s in enumerate(["TONIGHT:", "DEMOSPLASH 0x10"]):
        w = int(d.textlength(s, font=f))
        d.text(((128 - w) // 2, 3 + i * 18), s, font=f, fill=255)
    a = (np.array(im) > 127).astype(np.int8)
    a[20:] *= 2
    return a

LED = led_bitmap()
LED_COL = {1: np.array([1.0, 0.30, 0.06]), 2: np.array([1.0, 0.62, 0.10])}

def materials(kind, P, Nn, Dn, pr, alb, emit, spec, foot, t):
    f = STATE["f"]
    m = kind == K_LED
    if m.any():
        p = P[m]
        b = EXTRA["board"]
        x = p[:, 0] - BX
        u = np.where(Dn[m][:, 2] < 0, (b["cx"] + b["w"] / 2) - x, x - (b["cx"] - b["w"] / 2))
        u = u - (b["w"] - 128 * 0.0725) / 2
        v = (b["y0"] + b["h"]) - p[:, 1]
        col, row = np.floor(u / 0.0725), np.floor(v / 0.0725)
        fu, fv = u / 0.0725 - col, v / 0.0725 - row
        ok = (col >= 0) & (col < 128) & (row >= 0) & (row < 40)
        code = np.where(ok, LED[np.clip(row, 0, 39).astype(int), np.clip(col, 0, 127).astype(int)], 0)
        fw = foot[m] / 0.0725
        dot = T.fband(fu, 0.5, 0.36, fw) * T.fband(fv, 0.5, 0.36, fw)
        c = np.where((code == 1)[:, None], LED_COL[1], LED_COL[2])
        lit = code > 0
        emit[m] = c * (dot * np.where(lit, 9.0, 0.06))[:, None] + c * (lit * 0.5 * np.clip(fw - 0.5, 0, 1))[:, None]
        alb[m] = 0.02
    m = kind == K_PLATFORM
    if m.any():
        p = P[m]; top = Nn[m][:, 1] > 0.5
        x = p[:, 0] - BX
        e = np.where(x < MID, np.abs(x - (-2.6)), np.abs(x - (X2 + 2.6)))
        edge = np.where(x < MID, np.abs(x - (-1.85)), np.abs(x - (X2 + 1.85)))
        strip = T.fband(e, 0.0, 0.16, foot[m]) * top
        white = T.fband(edge, 0.0, 0.08, foot[m]) * top
        a = alb[m] * (0.85 + 0.3 * T.fbm(p[:, 0] * 0.8, p[:, 2] * 0.8, 2))[:, None]
        a = a * (1 - strip[:, None]) + np.array([0.85, 0.65, 0.05]) * strip[:, None]
        alb[m] = a * (1 - white[:, None]) + np.array([0.8, 0.8, 0.78]) * white[:, None]
    m = kind == K_TUBE
    if m.any():
        emit[m] = alb[m] * 9.0
    m = kind == K_EMITX
    if m.any():                                           # the stadium's lit bands and canopy edge, on the beat
        emit[m] = alb[m] * pr[m][:, 0:1] * 3.0 * pulse(f, 0.15)
    m = kind == K_DSCREEN
    if m.any():                                           # the callouts: clips and signs
        p = P[m] - np.array([BX, 0.0, 0.0])
        sid = pr[m][:, 0].astype(int)
        out = np.zeros((m.sum(), 3))
        for i in np.unique(sid):
            sc = SCREENS[i]
            q = sid == i
            d = p[q] - sc["c"]
            a = np.clip((d @ sc["r"]) / sc["w"] + 0.5, 0, 0.999)
            b = np.clip(0.5 - (d @ sc["u"]) / sc["h"], 0, 0.999)
            if sc["kind"] == "clip":
                fr = CO.clip_frame(sc["demo"], f + i * 7)
                tex = fr[(b * 120).astype(int), (a * 160).astype(int)]
                gx, gy = (a * 160) % 1.0, (b * 120) % 1.0      # the LED grid
                grid = 0.75 + 0.25 * (np.abs(gx - 0.5) < 0.38) * (np.abs(gy - 0.5) < 0.38)
                out[q] = tex * grid[:, None] * 2.2
            else:
                ban, _ = CO.banner(sc["demo"], 256, 96)
                out[q] = ban[(b * 96).astype(int), (a * 256).astype(int)] * 1.8
        emit[m] = out
        alb[m] = 0
    m = kind == K_PANEL
    if m.any():                                           # the panel: drawn per frame (the key, the button)
        p = P[m]
        u = np.clip((p[:, 2] - BX * 0 - PANEL["z0"]) / (PANEL["z1"] - PANEL["z0"]), 0, 0.999)
        v = np.clip((PANEL["y1"] - p[:, 1]) / (PANEL["y1"] - PANEL["y0"]), 0, 0.999)
        tex = panel_tex(f)
        th, tw = tex.shape[:2]
        c = tex[(v * th).astype(int), (u * tw).astype(int)]
        emit[m] = c * 1.15
        alb[m] = c * 0.15
    m = kind == K_STADSKIN
    if m.any():                                           # the bowl's outside as an LED skin: the callouts scroll
        p = P[m]
        q = to_aud(p)
        ang = np.arctan2(q[:, 1], q[:, 0])
        s_ = ang * 125.0                                  # metres round the wall
        y = q[:, 2]
        mq = CO.marquee()
        L = mq.shape[1]
        img_ = np.zeros(len(y))
        for (y0, y1, dirn) in ((11.0, 19.4, 1), (22.2, 30.6, -1)):
            row = (y1 - y) / (y1 - y0) * 16
            col = (s_ / ((y1 - y0) / 16) + dirn * f * 3.0) % L
            ok = (row >= 0) & (row < 16)
            r_, c_ = np.clip(row, 0, 15.99).astype(int), col.astype(int) % L
            dot = (np.abs(row % 1 - 0.5) < 0.35) & (np.abs(col % 1 - 0.5) < 0.35)
            img_ = np.maximum(img_, ok * mq[r_, c_] * (0.4 + 0.6 * dot))
        hue = ang * 3 + f * 0.05
        c = np.stack([0.5 + 0.5 * np.sin(hue), 0.5 + 0.5 * np.sin(hue + 2.1), 0.5 + 0.5 * np.sin(hue + 4.2)], 1)
        emit[m] = c * (img_ * 5.5 * pulse(f, 0.6))[:, None]
        alb[m] = alb[m] * 0.3
    if STATE.get("night", 0) > 0.3:                       # the town's windows breathe with the beat
        tw = (kind == T.K_CHOUSE) | (kind == T.K_BUILD) | (kind == T.K_HOUSE)
        emit[tw] *= pulse(f, 0.55)
    m = kind == K_NAV
    if m.any():
        code = pr[m][:, 0]
        strobe = 1.0 if (int(f) % 12) < 2 else 0.0
        k = np.where(code == 2, 40.0 * strobe, np.where(code == 3, 14.0, 14.0))
        emit[m] = alb[m] * k[:, None]
    m = kind == K_DRONE
    if m.any():
        spec[m] = T.reflect(Dn[m], Nn[m])[:, 1:2].clip(0, 1) * 0.15   # a little sheen on its shell
    m = kind == K_BHULL
    if m.any():                                           # the hull: dark plates, grime, a dull sheen
        p = P[m] - (blimp_pos(f) + np.array([BX, 0.0, 0.0]))
        plate = T.fstripe(p[:, 0], 3.1, 0.03, foot[m]) + T.fstripe(p[:, 1] + p[:, 2], 2.3, 0.03, foot[m])
        alb[m] = alb[m] * (0.7 + 0.6 * T.fbm(p[:, 0] * 0.15, (p[:, 1] + p[:, 2]) * 0.2, 3))[:, None] * \
            (1 - 0.4 * np.clip(plate, 0, 1))[:, None]
        nn_ = Nn[m]
        under = np.clip(-nn_[:, 1], 0, 1)                 # lit from below by the town and the stadium
        emit[m] = alb[m] * (np.array([1.4, 0.8, 1.0]) * under[:, None] * 0.35)
        pw = P[m]
        for k in range(len(UPLIGHTS)):                    # the stadium's lights aimed up at it: pools on the plates
            o, q = uplight_world(k, f)
            dv = q - o; dv = dv / np.linalg.norm(dv)
            rel = pw - q
            across = rel - (rel @ dv)[:, None] * dv[None, :]
            r2 = np.sum(across * across, 1)
            facing = np.clip(-(nn_ @ dv), 0, 1)
            pool = np.exp(-r2 / 30.0) * facing * (0.6 + 0.4 * beat(f))
            emit[m] += alb[m] * 0 + np.array([0.9, 0.95, 1.0]) * (pool * 3.0)[:, None] * (alb[m].mean(1) * 6)[:, None]
        spec[m] = alb[m] * 0.4 * (1 - np.abs(np.sum(Dn[m] * nn_, 1)))[:, None] ** 3   # a dull rim
    m = kind == K_BFRAME
    if m.any():
        emit[m] = alb[m] * (4.0 + 0.6 * math.sin(f * 0.4))
    m = kind == K_BSCREEN
    if m.any():
        p = (P[m] - (blimp_pos(f) + np.array([BX, 0.0, 0.0]))) / BSC - BS["c"]
        a = p[:, 0] / BS["w"] + 0.5
        b = (p @ BS["up"]) / BS["h"] + 0.5
        seg = int((f + 6) // 30) % 3                       # DEMOSPLASH 0x10, then a demo, then another
        reading = f >= F_CHASE and abs(cam_dist(f - F_CHASE) - D_READ) < 60.0   # our name while we read it
        if VOL_ON <= f < VOL_OFF or VOL_OFF + CUT_N <= f < VOL_OFF + CUT_N + JOIN:   # the volume: 10, the 11 appearing, the click, VOL 11 (held through the join)
            emit[m] = vol_screen(np.clip(a, 0, 1), np.clip(b, 0, 1), f)
        elif seg == 0 or f < F_STOP or reading:
            emit[m] = blimp_screen(np.clip(a, 0, 1), np.clip(b, 0, 1), f)
        else:
            i = (int((f + 6) // 90) * 2 + seg) % len(CO.ENTRIES)
            fr = CO.clip_frame(i, f)
            emit[m] = fr[(np.clip(1 - b, 0, 0.999) * 120).astype(int), (np.clip(a, 0, 0.999) * 160).astype(int)] * 2.4
        alb[m] = 0
    m = kind == K_SCREEN
    if m.any():
        p = P[m]
        a_, b_ = (p[:, 2] - AUD_C[2]) * 0.04, (p[:, 1] - AUD_C[1]) * 0.04
        tt = f * 0.15
        v = np.sin(a_ * 2 + tt) + np.sin(b_ * 3 - tt * 1.3) + np.sin((a_ + b_) * 1.5 + tt * 0.7)
        emit[m] = np.stack([0.5 + 0.5 * np.sin(v * 2.0), 0.5 + 0.5 * np.sin(v * 2.0 + 2.1),
                            0.5 + 0.5 * np.sin(v * 2.0 + 4.2)], 1) * 3.0
        alb[m] = 0

# ---- light from the lamps ---------------------------------------------------------------------------------------

def lights(P, Nn, cam, mask, kind):
    out = np.zeros(P.shape, np.float32)
    n = STATE.get("night", 0.0)
    idx = np.nonzero(mask)
    if len(idx[0]) == 0 or n < 0.05:
        return out
    p, nn = P[idx], Nn[idx]
    acc = np.zeros_like(p)
    for xl in (-10.5 + 2.2, -1.75 - 2.2, X2 + 1.75 + 2.2, X2 + 10.5 - 2.2):   # the canopies' tubes
        zc = np.clip(p[:, 2], PL0, PL1)
        v = np.stack([xl + BX - p[:, 0], 7.6 - p[:, 1], zc - p[:, 2]], 1)
        d2 = np.sum(v * v, 1)
        ndl = np.clip(np.sum(v * nn, 1) / np.sqrt(d2 + 1e-6), 0, 1)
        acc += (1.6 * ndl / (np.sqrt(d2) + 1.5) / (1 + np.abs(p[:, 2] - zc) * 0.15))[:, None] * np.array([1.0, 0.97, 0.9])
    L = EXTRA["lamps"]
    if len(L):
        Lw = L + np.array([BX, 0.0, 0.0])
        Lw = Lw[np.abs(Lw[:, 2] - cam.z) < 500]
        for q in Lw:
            sel = np.abs(p[:, 2] - q[2]) < 40
            if not sel.any():
                continue
            v = q - p[sel]
            d2 = np.sum(v * v, 1)
            ndl = np.clip(np.sum(v * nn[sel], 1) / np.sqrt(d2), 0, 1)
            acc[sel] += (14.0 * ndl / (d2 + 4.0))[:, None] * np.array([1.0, 0.82, 0.55])
    hp_list = [np.array([BX + hx, 0.7, (za(STATE["f"]) + 4.4)]) for hx in (-0.9, 0.9)]   # our headlights
    fwd = T.nrm([0, -0.05, 1])
    for hp in hp_list:
        v = hp - p
        d2 = np.sum(v * v, 1)
        l = v / np.sqrt(d2)[:, None]
        cone = np.clip(-(l @ fwd), 0, 1) ** 30
        acc += (90.0 * cone * np.clip(np.sum(l * nn, 1), 0, 1) / (d2 + 1))[:, None] * np.array([0.9, 0.95, 1.0])
    q = to_aud(p)                                         # the bowl: washed by its rig, a colour a beat, pulsing
    inside = ((q[:, 0] / 134.6) ** 2 + (q[:, 1] / 114.6) ** 2 < 0.97 ** 2) & (q[:, 2] < 32.0)   # the tiers, the pitch
    if inside.any():
        f = STATE["f"]
        tt = (f - F_HAND) / T.FPS + T_H
        k = int(math.floor(tt * T.FPS / (2 * BEAT_F)))   # a colour a pulse
        pal = np.array([[1.0, 0.25, 0.8], [0.25, 0.8, 1.0], [1.0, 0.7, 0.2], [0.5, 0.35, 1.0]])
        col = pal[k % 4]
        up = 0.45 + 0.55 * np.clip(nn[inside, 1], 0, 1)
        acc[inside] += (up * (0.25 + 1.6 * beat(f)))[:, None] * col
    out[idx] = acc * n
    return out

# ---- after shading: the stadium's beams and lasers, from their lights; the windscreen as we pass through ---------

def beam(img, IZ, cam, p0, p1, w0, w1, col, inten, segs=24):
    H, W = img.shape[:2]
    pts = np.linspace(0, 1, segs + 1)[:, None] * (p1 - p0) + p0
    sx, sy, sz = cam.project(pts)
    for i in range(segs):
        if sz[i] < T.NEAR or sz[i + 1] < T.NEAR:
            continue
        ax, ay, bx_, by_ = sx[i], sy[i], sx[i + 1], sy[i + 1]
        u0, u1 = i / segs, (i + 1) / segs
        wa = max((w0 + (w1 - w0) * u0) * cam.f / sz[i], 0.6)
        wb = max((w0 + (w1 - w0) * u1) * cam.f / sz[i + 1], 0.6)
        r = max(wa, wb) * 2
        x0 = max(0, int(min(ax, bx_) - r)); x1 = min(W, int(max(ax, bx_) + r) + 1)
        y0 = max(0, int(min(ay, by_) - r)); y1 = min(H, int(max(ay, by_) + r) + 1)
        if x0 >= x1 or y0 >= y1 or (x1 - x0) * (y1 - y0) > 600000:
            continue
        px = np.arange(x0, x1, dtype=np.float32)[None, :] + 0.5
        py = np.arange(y0, y1, dtype=np.float32)[:, None] + 0.5
        dx, dy = bx_ - ax, by_ - ay
        L2 = dx * dx + dy * dy + 1e-9
        ur = ((px - ax) * dx + (py - ay) * dy) / L2
        u = np.clip(ur, 0, 1)
        d = np.hypot(px - (ax + u * dx), py - (ay + u * dy))
        w = wa + (wb - wa) * u
        iz = 1 / (sz[i] + (sz[i + 1] - sz[i]) * u)
        inner = ((ur >= 0) | (i == 0)) & ((ur < 1) | (i == segs - 1))   # no caps where segments meet: no beads
        vis = (iz >= IZ[y0:y1, x0:x1] * 0.999) & inner
        along_ = 1 - (u0 + (u1 - u0) * u)
        a = np.exp(-(d / w) ** 2) * vis * along_ ** 1.5 * inten / np.maximum(w, 1.0) ** 0.3
        img[y0:y1, x0:x1] += a[..., None] * col
    return img

def aud_world(P):
    return to_film(P) + np.array([BX, 0.0, 0.0])

def glass(img, cam):
    """Through the windscreen: as the eye nears the glass, the cab's lights reflected in it and a tint; at the
    moment it passes, the glass's edge sweeping past."""
    z = cam.eye[2] - ZS
    k = smooth(0.0, 1.0, z) * (1 - smooth(1.2, 2.0, z))   # in the glass's reach
    if k <= 0:
        return img
    H, W = img.shape[:2]
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32)
    u, v = xs / W, ys / H
    refl = np.exp(-((u - 0.28) ** 2 / 0.004 + (v - 0.78) ** 2 / 0.01))[..., None] * np.array([1.0, 0.6, 0.25]) * 0.8
    refl = refl + np.exp(-((u - 0.7) ** 2 / 0.002 + (v - 0.82) ** 2 / 0.006))[..., None] * np.array([0.3, 0.8, 1.0]) * 0.5
    streak = np.exp(-((u - v * 0.4 - 0.25) ** 2) / 0.0015)[..., None] * 0.12
    img = img * (1 - 0.18 * k) + (refl + streak) * k
    edge_y = H * (1.05 - 1.2 * smooth(1.2, 1.9, z))       # the glass's top edge sweeping down past us
    e = np.exp(-((ys - edge_y) / (H * 0.012)) ** 2)[..., None] * smooth(1.0, 1.4, z) * (1 - smooth(1.8, 2.0, z))
    return img + e * np.array([0.9, 0.95, 1.0]) * 1.5

def key_value(f):
    """The key: at 10, then one click to 11 (a short decisive turn, a hair of overshoot)."""
    if f < F_KEY:
        return 10.0
    return 11.0 + 0.12 * math.exp(-(f - F_KEY) * 1.5) * (f - F_KEY < 3)

def button_depth(f):
    """The button: out, then a hard push in, staying in."""
    return 0.0 if f < F_BTN else (0.6 if f < F_BTN + 1 else 1.0)

_PT = {}

def lamp_on(f):
    """MAX OVERCLOCK: blinking from the button's push, a slow warning blink: on for a beat, off for a beat."""
    return f >= F_BTN + 2 and ((int(f) - F_BTN - 2) % 12) < 6

def panel_tex(f):
    """The box's face, 600 x 450 (1 x 0.75 m), in its red housing: BREAK GLASS across the top; an oversized key
    switch (a cartoon of one: 0-9 small round it, the 10 and the red 11 huge); the big red mushroom button on its
    yellow ring; the MAX OVERCLOCK lamp under them."""
    from PIL import Image, ImageDraw, ImageFont
    kv, bd, lo = key_value(f), button_depth(f), lamp_on(f)
    key = (round(kv, 2), bd, lo)
    if key in _PT:
        return _PT[key]
    W_, H_ = 600, 450
    im = Image.new("RGB", (W_, H_), (200, 20, 16))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle([16, 16, W_ - 17, H_ - 17], 12, fill=(236, 234, 228))
    F = lambda n: ImageFont.truetype(CO.HEL, n, index=1)
    t = "BREAK GLASS"
    d.text(((W_ - d.textlength(t, font=F(44))) / 2, 24), t, font=F(44), fill=(200, 20, 16))
    cx, cy, R = 175, 230, 70                              # the key switch, oversized
    for v_ in range(12):
        a = math.radians(-135 + 270 * v_ / 11)
        big = v_ >= 10
        col = (220, 20, 16) if v_ == 11 else (30, 30, 30)
        x0, y0 = cx + math.sin(a) * R * 1.12, cy - math.cos(a) * R * 1.12
        x1, y1 = cx + math.sin(a) * R * (1.42 if big else 1.28), cy - math.cos(a) * R * (1.42 if big else 1.28)
        d.line([(x0, y0), (x1, y1)], fill=col, width=7 if big else 3)
        if big:
            fs = F(54 if v_ == 11 else 40)
            tx, ty = cx + math.sin(a) * R * 1.85, cy - math.cos(a) * R * 1.85
        else:
            fs = F(17)
            tx, ty = cx + math.sin(a) * R * 1.55, cy - math.cos(a) * R * 1.55
        s_ = str(v_)
        d.text((tx - d.textlength(s_, font=fs) / 2, ty - fs.size * 0.6), s_, font=fs, fill=col)
    d.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(160, 160, 166), outline=(80, 80, 84), width=4)
    d.ellipse([cx - R * 0.7, cy - R * 0.7, cx + R * 0.7, cy + R * 0.7], fill=(70, 70, 74))
    a = math.radians(-135 + 270 * kv / 11)                # the key: a big brass bow, a black pointer
    ux, uy = math.sin(a), -math.cos(a)
    px, py = -uy, ux
    L1, wd = R * 1.0, 20
    d.polygon([(cx + ux * L1 + px * wd, cy + uy * L1 + py * wd), (cx + ux * L1 - px * wd, cy + uy * L1 - py * wd),
               (cx - ux * L1 - px * wd, cy - uy * L1 - py * wd), (cx - ux * L1 + px * wd, cy - uy * L1 + py * wd)],
              fill=(222, 182, 66), outline=(110, 80, 20), width=3)
    d.polygon([(cx + ux * L1 * 1.05, cy + uy * L1 * 1.05), (cx + ux * L1 * 0.55 + px * 15, cy + uy * L1 * 0.55 + py * 15),
               (cx + ux * L1 * 0.55 - px * 15, cy + uy * L1 * 0.55 - py * 15)], fill=(25, 25, 25))
    d.ellipse([cx - 10, cy - 10, cx + 10, cy + 10], fill=(110, 80, 20))
    t = "VOLUME"
    d.text((cx - d.textlength(t, font=F(20)) / 2, cy + R * 1.45), t, font=F(20), fill=(30, 30, 30))
    bx, by, BR = 440, 225, 76                             # the mushroom button
    d.ellipse([bx - BR - 14, by - BR - 14, bx + BR + 14, by + BR + 14], fill=(240, 196, 0), outline=(20, 20, 20), width=3)
    sh = 12 * (1 - bd)
    d.ellipse([bx - BR + 7, by - BR + 12, bx + BR + 7, by + BR + 12], fill=(60, 20, 16))
    r_ = BR * (1 - 0.08 * bd)
    col = (220, 20, 16) if bd < 0.5 else (165, 10, 8)
    d.ellipse([bx - r_ - sh * 0.3, by - r_ - sh, bx + r_ - sh * 0.3, by + r_ - sh], fill=col, outline=(120, 6, 4), width=3)
    d.ellipse([bx - r_ * 0.6, by - r_ * 0.75 - sh, bx + r_ * 0.1, by - r_ * 0.2 - sh],
              fill=(255, 130, 120) if bd < 0.5 else (205, 60, 50))
    ly = 385                                              # MAX OVERCLOCK
    d.ellipse([60, ly - 26, 112, ly + 26], fill=(255, 150, 20) if lo else (90, 50, 20), outline=(20, 20, 20), width=3)
    t = "MAX OVERCLOCK"
    d.text((130, ly - 26), t, font=F(46), fill=(200, 20, 16))
    a_ = (np.array(im, np.float32) / 255) ** 2.2
    if lo:
        yy, xx = np.mgrid[0:H_, 0:W_]
        g = np.exp(-((xx - 86) ** 2 + (yy - ly) ** 2) / 600.0)
        a_ = a_ + g[..., None] * np.array([4.0, 2.0, 0.3])
    _PT[key] = a_
    return a_

def _pane_quad():
    """The box's glass front, in the film's world."""
    off = np.array([BX, 0.0, 0.0])
    x = PANEL["x"] + 0.075
    y0, y1, z0, z1 = PANEL["y0"] + 0.04, PANEL["y1"] - 0.04, PANEL["z0"] + 0.04, PANEL["z1"] - 0.04
    return np.array([(x, y1, z0), (x, y1, z1), (x, y0, z1), (x, y0, z0)]) + off

def cover(img, cam):
    """The glass front of the box until it breaks: a faint tint, the platform's light streaking across it."""
    f = STATE["f"]
    if not (F_GO - 2 <= f < F_SMASH + 0.5):
        return img
    from PIL import Image, ImageDraw
    H, W = img.shape[:2]
    sx, sy, sz = cam.project(_pane_quad())
    if (sz < 0.05).any():
        return img
    lay = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    dr = ImageDraw.Draw(lay)
    pts = list(zip(sx, sy))
    dr.polygon(pts, fill=(210, 230, 255, 40))
    lw = max(1, int(3 * W / 1536))
    a, b, c, d_ = [np.array(p) for p in pts]
    for u0, u1, al in ((0.15, 0.55, 90), (0.62, 0.8, 50)):   # streaks of the tubes' light
        p0, p1 = a + (b - a) * u0, a + (b - a) * u1
        q0, q1 = d_ + (c - d_) * (u0 - 0.12), d_ + (c - d_) * (u1 - 0.12)
        dr.polygon([tuple(p0), tuple(p1), tuple(q1), tuple(q0)], fill=(255, 255, 255, al // 3))
    dr.line(pts + [pts[0]], fill=(240, 245, 255, 160), width=lw)
    al = np.array(lay, np.float32) / 255
    return img * (1 - al[..., 3:4] * 0.35) + al[..., :3] * al[..., 3:4] * 1.3

_SH = {}

def shatter(img, cam):
    """The box's glass breaking by itself: cracks racing out from a point, then the pane bursting outward in shards
    that fly at us, their edges catching the platform's light."""
    f = STATE["f"]
    age = f - F_SMASH
    if f < F_CRACK or age > 12:
        return img
    from PIL import Image, ImageDraw
    H, W = img.shape[:2]
    sx, sy, sz = cam.project(_pane_quad())
    if (sz < 0.05).any():
        return img
    if "shards" not in _SH:
        rng = np.random.default_rng(1982)
        n = 14
        angs = np.sort((np.arange(n) + rng.uniform(-0.3, 0.3, n)) / n * 2 * np.pi)
        radii = [0.0, 0.06, 0.14, 0.26, 0.42, 0.62, 0.9]
        jit = rng.uniform(0.85, 1.15, (len(radii), n))
        pt = lambda i, j: np.array([math.cos(angs[j % n]), math.sin(angs[j % n]) * 0.8]) * radii[i] * jit[i, j % n]
        shards = []
        for i in range(len(radii) - 1):
            for j in range(n):
                poly = [pt(i, j), pt(i, j + 1), pt(i + 1, j + 1), pt(i + 1, j)]
                shards.append((poly, sum(poly) / 4, rng.uniform(-0.6, 0.6), rng.uniform(0.7, 1.4)))
        _SH.update(shards=shards, angs=angs, radii=radii, jit=jit)
    cx, cy = sx.mean() + (sx.max() - sx.min()) * 0.12, sy.mean() - (sy.max() - sy.min()) * 0.1
    S = (sx.max() - sx.min()) * 0.55
    clip = (sx.min(), sy.min(), sx.max(), sy.max())
    lay = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    dr = ImageDraw.Draw(lay)
    lw = max(1, int(2 * W / 1536))
    inside = lambda p: (min(max(p[0], clip[0]), clip[2]), min(max(p[1], clip[1]), clip[3]))
    if age < 0:                                           # cracks racing out across the pane
        grow = (f - F_CRACK) / (F_SMASH - F_CRACK)
        angs, radii, jit = _SH["angs"], _SH["radii"], _SH["jit"]
        n = len(angs)
        for j in range(n):
            pts = []
            for i, r in enumerate(radii):
                if r > grow * 1.0 + 0.05:
                    break
                v = np.array([math.cos(angs[j]), math.sin(angs[j]) * 0.8]) * r * jit[i, j]
                pts.append(inside((cx + v[0] * S, cy + v[1] * S)))
            if len(pts) > 1:
                dr.line(pts, fill=(245, 248, 255, 230), width=lw)
        for i, r in enumerate(radii[1:4], 1):
            if r < grow * 0.7:
                ring = [inside((cx + math.cos(angs[j % n]) * r * jit[i, j % n] * S,
                                cy + math.sin(angs[j % n]) * 0.8 * r * jit[i, j % n] * S)) for j in range(n + 1)]
                dr.line(ring, fill=(245, 248, 255, 160), width=lw)
        dr.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=(255, 255, 255, 240))
    else:                                                 # the pane bursting out, the shards flying at us
        for poly, c, rot, sp in _SH["shards"]:
            k = 1 + age * 0.55 * sp * (0.5 + np.linalg.norm(c))
            ca, sa = math.cos(rot * age * 0.5), math.sin(rot * age * 0.5)
            out = []
            for q in poly:
                q2 = (q - c) * (1 + age * 0.18)
                q2 = np.array([ca * q2[0] - sa * q2[1], sa * q2[0] + ca * q2[1]])
                pos = c * k + q2 + np.array([0, 0.02 * age * age])
                out.append((cx + pos[0] * S, cy + pos[1] * S))
            al = max(0.0, 1 - age / 12.0)
            dr.polygon(out, fill=(205, 225, 255, int(45 * al)))
            dr.line(out + [out[0]], fill=(255, 248, 230, int(235 * al)), width=lw + (1 if age < 5 else 0))
    a = np.array(lay, np.float32) / 255
    return img * (1 - a[..., 3:4] * 0.3) + a[..., :3] * a[..., 3:4] * 2.5

def blimp_post(img, cam, IZ, f, n):
    """The airship's running lights (hundreds, warm; a string round the screen; red beacons blinking on the
    masts) and its searchlights raking down through the haze."""
    H, W = img.shape[:2]
    pos = blimp_pos(f) + np.array([BX, 0.0, 0.0])
    pts, kinds = BLIMP_LIGHTS
    P = pts * BSC + pos
    sx, sy, sz = cam.project(P)
    ok = (sz > 1) & (sx >= 1) & (sx < W - 1) & (sy >= 1) & (sy < H - 1)
    ix, iy = sx[ok].astype(int), sy[ok].astype(int)
    vis = 1 / sz[ok] >= IZ[iy, ix] * 0.995
    k = kinds[ok]
    blink = np.where(k == 2, (np.sin(f * 0.9 + np.arange(len(k))) > 0.3).astype(float), 1.0)
    rgb = np.where((k == 2)[:, None], np.array([4.0, 0.2, 0.1]), np.where((k == 1)[:, None], np.array([3.5, 2.2, 0.9]),
                                                                          np.array([3.0, 1.7, 0.7])))
    s = np.clip(120.0 / sz[ok], 0.9, 4.0)[:, None] * (0.4 + 0.8 * n) * blink[:, None]   # strong against the sky
    sel = vis
    for dx, dy, wgt in ((0, 0, 1.0), (1, 0, 0.35), (-1, 0, 0.35), (0, 1, 0.35), (0, -1, 0.35)):
        np.add.at(img, (iy[sel] + dy, ix[sel] + dx), rgb[sel] * s[sel] * wgt)
    for k in range(len(UPLIGHTS)):                        # the stadium's lights aimed up at it, through the haze
        o, q = uplight_world(k, f)
        img = beam(img, IZ, cam, o, q + (q - o) * 0.02, 0.5, 5.0, np.array([0.85, 0.9, 1.0]),
                   0.45 * n * (0.55 + 0.45 * beat(f)))
    for k_, (lx, lz) in enumerate(((-30, -6), (-10, 5), (12, -4), (32, 6))):      # searchlights from the belly
        o = pos + np.array([lx, -17.0, lz]) * BSC
        sw = 0.5 * math.sin(f * 0.035 + k_ * 1.7)
        d = np.array([math.sin(sw) * 0.5, -1.0, 0.35 * math.cos(f * 0.03 + k_)])
        d = d / np.linalg.norm(d)
        img = beam(img, IZ, cam, o, o + d * 260.0, 0.8, 16.0, np.array([0.55, 0.85, 1.0]), 0.5 * (0.3 + 0.7 * n))
    return img

def post(img, cam, IZ, Z, t):
    img = cover(img, cam)
    img = shatter(img, cam)
    n = STATE.get("night", 0.0)
    f = STATE["f"]
    img = blimp_post(img, cam, IZ, f, n)
    au = audience()
    js = _AU.get("json")
    if n < 0.2 or js is None:
        return img
    tt = (f - F_HAND) / T.FPS + T_H                       # their show's clock: T_H at the hand-off
    lights_ = js["lights"]
    for l in [l for l in lights_ if l["type"] == "search"]:
        o = np.array(l["pos"], float)
        u = au.search_dir(l, tt) if au else np.array([0, 0, 1.0])
        img = beam(img, IZ, cam, aud_world(o), aud_world(o + u * 900), 0.6, 18.0,
                   np.array(l["rgb"]) * l.get("power", 1.0), 0.6 * n * pulse(f, 0.25))
    for l in [l for l in lights_ if l["type"] == "spot"]:
        o = np.array(l["pos"], float)
        u = au.spot_dir(l, tt) if au else np.array([0, -0.5, -1.0])
        L = (o[2] - 1.8) / max(-u[2], 0.05)
        img = beam(img, IZ, cam, aud_world(o), aud_world(o + u * L), 0.3, 4.0, np.array(l["rgb"]), 0.45 * n * pulse(f, 0.1))
    if au:
        for l in [l for l in lights_ if l["type"] == "lasers"]:
            src = np.array(l["pos"], float)
            dirs, on = au.laser_beams(l, tt)
            for dvec, o in zip(dirs, on):
                if o > 0.05:
                    img = beam(img, IZ, cam, aud_world(src), aud_world(src + dvec * 125.0), 0.05, 0.25,
                               np.array(l["rgb"]) * 1.4, 1.2 * n * o * pulse(f, 0.2), segs=8)
    img = balloons(img, cam, IZ, f, n)
    img = fireworks(img, cam, IZ, f, n)
    img = photo_flashes(img, cam, IZ, f, n)
    return img

def photo_flashes(img, cam, IZ, f, n):
    """People taking pictures: a few dozen flashes a frame, anywhere in the bowl, each one frame, a colour each (the
    stadium design does the same in the crowd); off the beat, whenever."""
    if n < 0.5:
        return img
    H, W = img.shape[:2]
    rng = np.random.default_rng(int(round(f)) * 131 + 7)
    N = 36
    r = rng.uniform(0.12, 0.93, N)
    a = rng.uniform(0, 2 * np.pi, N)
    z = np.where(r < 0.62, 1.7, 1.7 + (r - 0.62) / 0.33 * 28.0)
    P = aud_world(np.stack([134.6 * r * np.cos(a), 114.6 * r * np.sin(a), z], 1))
    hue = rng.uniform(0, 1, (N, 3)) ** 2
    col = 0.3 + 0.7 * hue / hue.max(1, keepdims=True)
    sx, sy, sz = cam.project(P)
    for i in range(N):
        if sz[i] < 2 or not (0 <= sx[i] < W and 0 <= sy[i] < H):
            continue
        if 1 / sz[i] < IZ[int(sy[i]), int(sx[i])] * 0.97:
            continue
        rp = min(max(cam.f * 0.45 / sz[i], 1.2), 9.0)
        x0, x1 = max(0, int(sx[i] - 4 * rp)), min(W, int(sx[i] + 4 * rp) + 1)
        y0, y1 = max(0, int(sy[i] - 4 * rp)), min(H, int(sy[i] + 4 * rp) + 1)
        px = np.arange(x0, x1)[None, :] + 0.5 - sx[i]
        py = np.arange(y0, y1)[:, None] + 0.5 - sy[i]
        g = np.exp(-(px * px + py * py) / (rp * rp)) * 9.0 + np.exp(-(px * px + py * py) / (9 * rp * rp)) * 0.8
        img[y0:y1, x0:x1] += g[..., None] * col[i]
    return img

# ---- balloons rising into the night, fireworks over the stadium -------------------------------------------------

_BAL = {}

def balloons(img, cam, IZ, f, n):
    """Hundreds of balloons floating up from the stadium and the town, at all depths, dark but for the beams and
    the stadium's light catching them on the beat."""
    if "p" not in _BAL:
        rng = np.random.default_rng(99)
        N = 900
        c = AUD_C + np.array([BX, 0.0, 0.0])
        r = np.sqrt(rng.uniform(0, 1, N)) * 420
        a = rng.uniform(0, 2 * np.pi, N)
        _BAL["p"] = np.stack([c[0] + r * np.cos(a) - 80, rng.uniform(-60, 260, N), c[2] + r * np.sin(a) * 1.2 - 150], 1)
        _BAL["v"] = rng.uniform(0.18, 0.45, N)
        _BAL["c"] = np.array([[1, 0.15, 0.2], [1, 0.8, 0.1], [0.2, 0.5, 1], [0.95, 0.95, 0.95], [0.9, 0.2, 0.9],
                              [0.2, 0.9, 0.4]])[rng.integers(0, 6, N)]
        _BAL["ph"] = rng.uniform(0, 6.28, N)
    if n < 0.4:
        return img
    P = _BAL["p"].copy()
    P[:, 1] = T.LAND + (P[:, 1] + _BAL["v"] * f) % 320
    P[:, 0] += 1.5 * np.sin(f * 0.05 + _BAL["ph"])
    sx, sy, sz = cam.project(P)
    H, W = img.shape[:2]
    rad = 0.9 * cam.f / np.maximum(sz, 1e-3)
    ok = (sz > 2) & (sx > -rad) & (sx < W + rad) & (sy > -rad) & (sy < H + rad)
    lit = 0.6 + 1.2 * beat(f) * np.exp(-np.linalg.norm(P - (AUD_C + [BX, 40, 0]), axis=1) / 300)
    for i in np.nonzero(ok)[0]:
        R = rad[i]
        if R < 0.5:
            x, y = int(sx[i]), int(sy[i])
            if 0 <= x < W and 0 <= y < H and 1 / sz[i] >= IZ[y, x]:
                img[y, x] += _BAL["c"][i] * lit[i] * R * 1.2
            continue
        x0, x1 = max(0, int(sx[i] - R - 1)), min(W, int(sx[i] + R + 2))
        y0, y1 = max(0, int(sy[i] - R * 1.2 - 1)), min(H, int(sy[i] + R * 1.2 + 2))
        if x0 >= x1 or y0 >= y1:
            continue
        yy, xx = np.mgrid[y0:y1, x0:x1] + 0.5
        dx, dy = (xx - sx[i]) / R, (yy - sy[i]) / (R * 1.2)
        d2 = dx * dx + dy * dy
        cov = np.clip((1 - d2) * R, 0, 1) * (1 / sz[i] >= IZ[y0:y1, x0:x1])
        shade = 0.35 + 0.65 * np.clip(1 - ((dx + 0.35) ** 2 + (dy + 0.4) ** 2), 0, 1)
        hl = np.exp(-((dx + 0.35) ** 2 + (dy + 0.4) ** 2) / 0.02) * 2.0
        col = _BAL["c"][i][None, None] * (shade * lit[i])[..., None] + hl[..., None] * lit[i]
        img[y0:y1, x0:x1] = img[y0:y1, x0:x1] * (1 - cov[..., None]) + col * cov[..., None]
    return img

def fireworks(img, cam, IZ, f, n):
    """Shells bursting high over the stadium on the first beat of every second bar."""
    if n < 0.5:
        return img
    H, W = img.shape[:2]
    for k in range(4):
        start = F_HAND - 24 * k - ((F_HAND - f) // 24) * 24 - 24 * 0
        start = f - ((f - F_HAND) % 24) - 24 * k
        age = f - start
        if age < 0 or age > 30:
            continue
        rng = np.random.default_rng(int(start) * 7 + 3)
        for s in range(2):
            c0 = AUD_C + np.array([BX + rng.uniform(-120, 120), rng.uniform(140, 230), rng.uniform(-120, 120)])
            col = np.array([[1, 0.4, 0.2], [0.3, 0.6, 1], [1, 0.9, 0.3], [0.9, 0.3, 1], [0.3, 1, 0.5]][rng.integers(5)])
            d = rng.normal(size=(150, 3)); d /= np.linalg.norm(d, axis=1, keepdims=True)
            rr = 45 * (1 - math.exp(-age / 4.0))
            P = c0 + d * rr - np.array([0, 0.04 * age * age, 0])
            sx, sy, sz = cam.project(P)
            ok = (sz > 5) & (sx >= 1) & (sx < W - 1) & (sy >= 1) & (sy < H - 1)
            fade = max(0.0, 1 - age / 30.0) ** 1.5 * 6.0
            ix, iy = sx[ok].astype(int), sy[ok].astype(int)
            vis = 1 / sz[ok] >= IZ[iy, ix]
            for dx_, dy_, w_ in ((0, 0, 1.0), (1, 0, 0.4), (0, 1, 0.4)):
                np.add.at(img, (iy[vis] + dy_, ix[vis] + dx_), col * fade * w_)
    return img

# ---- running it ---------------------------------------------------------------------------------------------------

def install():
    T.HOOKS.update(build=build_hook, dynamic=dynamic, light=lights, post=post, materials=materials, vel=vel_hook)
    T.Cam = NightCam
    T.env_at = lambda z: T.DUSK
    T.exposure = exposure_night
    T.sky_dusk = sky_night
    T.train_front = lambda t: -1e9
    T.visible = visible_night
    T.TOWN0[0] = TOWN0
    T.scene = scene_night

OLD = {}

def render_old(f, W, H):
    """The frame as the ride's own world would have it, for the join (the night's new things dissolve in)."""
    saved = (T.STATIC, dict(T.HOOKS), T.scene, T.visible, T.TOWN0[0])
    T.STATIC = OLD
    T.HOOKS.clear()
    T.scene, T.visible, T.TOWN0[0] = ORIG_SCENE, ORIG_VISIBLE, 1e9
    try:
        return T.render(f, W, H, overlay=False)
    finally:
        T.STATIC, T.scene, T.visible, T.TOWN0[0] = saved[0], saved[2], saved[3], saved[4]
        T.HOOKS.update(saved[1])

KNOB_ON, KNOB_CLICK = F_STOP - 18, F_STOP + 2          # the cab's volume knob: seen at 10, one click to 11

def knob_value(f):
    """The knob's setting: 10 until the click, then 11 (a little overshoot the frame after, settling)."""
    if f < KNOB_CLICK:
        return 10.0
    if f < KNOB_CLICK + 1:
        return 10.6
    if f < KNOB_CLICK + 2:
        return 11.12
    return 11.0

def draw_knob(img, f, W, H):
    """The PA's volume knob on the driver's console, bottom right of the cab's view: a chunky black knob with a
    white pointer on a lit plate printed 0-11 (11 in red), the console's edge across the bottom. It slides down out of
    the frame as the camera moves forward through the windscreen."""
    from PIL import Image, ImageDraw, ImageFont
    if f < KNOB_ON - 6 or f > F_PUSH + 10:
        return img
    s_ = W / 1536.0
    ez = (cab_eye(f)[2] if f <= F_CHASE else chase_eye(f)[2]) - ZS if f >= F_PUSH else 0.0
    slide = (1 - ease((f - (KNOB_ON - 6)) / 6.0)) * 300 * s_ + max(0.0, ez) * 420 * s_   # up into view, then away
    fade = 1.0
    cx, cy, R = 1225 * s_, 560 * s_ + slide, 84 * s_
    lay = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(lay)
    top = 655 * s_ + slide                                # the console's edge
    d.polygon([(0, H), (0, top + 60 * s_), (W * 0.55, top + 20 * s_), (W, top - 10 * s_), (W, H)], fill=(18, 18, 20, 255))
    d.line([(0, top + 60 * s_), (W * 0.55, top + 20 * s_), (W, top - 10 * s_)], fill=(60, 60, 66, 255), width=max(1, int(3 * s_)))
    pr = R * 1.75                                         # the plate and its scale
    d.ellipse([cx - pr, cy - pr, cx + pr, cy + pr], fill=(38, 36, 34, 255), outline=(90, 88, 84, 255), width=max(1, int(2 * s_)))
    fnt = ImageFont.truetype(CO.HEL, max(8, int(25 * s_)), index=1)
    for v in range(12):
        a = math.radians(-125 + 250 * v / 11)
        x0, y0 = cx + math.sin(a) * R * 1.12, cy - math.cos(a) * R * 1.12
        x1, y1 = cx + math.sin(a) * R * 1.32, cy - math.cos(a) * R * 1.32
        col = (235, 40, 30, 255) if v == 11 else (235, 230, 220, 255)
        d.line([(x0, y0), (x1, y1)], fill=col, width=max(1, int((4 if v == 11 else 3) * s_)))
        tx, ty = cx + math.sin(a) * R * 1.55, cy - math.cos(a) * R * 1.55
        t = str(v)
        f2 = ImageFont.truetype(CO.HEL, max(8, int((31 if v == 11 else 22) * s_)), index=1)
        w_ = d.textlength(t, font=f2)
        d.text((tx - w_ / 2, ty - f2.size * 0.6), t, font=f2, fill=col)
    d.text((cx - d.textlength("VOLUME", font=fnt) / 2, cy + R * 1.05), "VOLUME", font=fnt, fill=(200, 196, 188, 255))
    d.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(14, 14, 15, 255), outline=(70, 70, 74, 255), width=max(1, int(3 * s_)))
    for k in range(36):                                   # the knurled edge
        a = 2 * math.pi * k / 36 + math.radians(250 * knob_value(f) / 11)
        d.line([(cx + math.sin(a) * R * 0.88, cy - math.cos(a) * R * 0.88), (cx + math.sin(a) * R, cy - math.cos(a) * R)],
               fill=(46, 46, 50, 255), width=max(1, int(2 * s_)))
    d.ellipse([cx - R * 0.7, cy - R * 0.7, cx + R * 0.7, cy + R * 0.7], fill=(26, 26, 28, 255))
    d.ellipse([cx - R * 0.55, cy - R * 0.85, cx + R * 0.15, cy - R * 0.2], fill=(60, 60, 64, 120))   # a highlight
    a = math.radians(-125 + 250 * knob_value(f) / 11)
    d.line([(cx + math.sin(a) * R * 0.25, cy - math.cos(a) * R * 0.25), (cx + math.sin(a) * R * 0.95, cy - math.cos(a) * R * 0.95)],
           fill=(250, 250, 245, 255), width=max(2, int(7 * s_)))
    if KNOB_CLICK <= f < KNOB_CLICK + 4:                  # the click: a little flash of the red 11
        a11 = math.radians(125)
        gx, gy = cx + math.sin(a11) * R * 1.55, cy - math.cos(a11) * R * 1.55
        rr = 28 * s_
        d.ellipse([gx - rr, gy - rr, gx + rr, gy + rr], outline=(255, 60, 40, 200), width=max(1, int(3 * s_)))
    a_ = np.array(lay, np.float32) / 255
    al = a_[..., 3:4] * fade
    return img * (1 - al) + a_[..., :3] * 255 * al

def render_new(f, W, H):
    set_light(f)
    return _render_new(f, W, H)

def _render_new(f, W, H):
    set_light(f)
    T.WIRE_BACK[0] = 0.0
    T.TRAIN_DZ[0] = 0.0
    return T.render(f, W, H, overlay=False)

_SC = {}

def render_theirs(f, W, H):
    """Their renderer, at our camera mapped into their frame (their clock 0 at the hand-off), 0..255."""
    au = audience()
    if "sc" not in _SC:
        _SC["sc"] = au.Scene()
    cam = NightCam(f, W, H)
    eye = to_aud(cam.eye)
    tgt = to_aud(cam.eye + cam.basis[2] * 100.0)
    return au.render(_SC["sc"], eye, tgt, (f - F_HAND) / T.FPS + T_H, W, H) * 255.0

def render(f, W, H):
    set_light(f)
    if f < XFADE:
        a = smooth(0.0, float(XFADE), f)
        old = render_old(f, W, H)
        return old if a <= 0 else old * (1 - a) + render_new(f, W, H) * a
    if F_GO <= f < F_VIEW:                                # the dissolve from the cab to the post
        a = smooth(float(F_GO), float(F_VIEW), f)
        FORCE_CAB[0] = True
        try:
            cab = render_new(f, W, H)
        finally:
            FORCE_CAB[0] = False
        return cab * (1 - a) + render_new(f, W, H) * a
    if f > F_HAND - DISSOLVE and audience() is not None:
        a = smooth(F_HAND - DISSOLVE, F_HAND, f) ** 0.6
        mine = render_new(f, W, H)
        if a <= 0:
            return mine
        return mine * (1 - a) + render_theirs(f, W, H) * a
    return render_new(f, W, H)

SS = int(os.environ.get("SS", "1"))

def their_frame(k, W, H):
    """Their film's frame k (from their cache if it's there at this size, else rendered)."""
    p = AUWORK + "frames_%s/%04d.rgb" % (os.environ.get("AUTAG", "v3"), k)
    if os.path.exists(p) and os.path.getsize(p) == W * H * 3:
        return np.fromfile(p, np.uint8).reshape(H, W, 3)
    au = audience()
    if "sc" not in _SC:
        _SC["sc"] = au.Scene()
    eye, tgt = au.camera(k / T.FPS)
    return np.clip(au.render(_SC["sc"], eye, tgt, k / T.FPS, W, H) * 255 + 0.5, 0, 255).astype(np.uint8)

def frame_bytes(args):
    k, W, H = args
    k = design_f(k)                                       # the film's frame k: the design's (the orbit skipped)
    install()
    if k > F_HAND:                                        # past the crest: their scene, on our path
        set_light(float(k))
        return np.clip(render_theirs(float(k), W, H), 0, 255).astype(np.uint8).tobytes()
    if k >= F_HAND - DISSOLVE:                            # their renderer works at the output size
        img = render(float(k), W, H) if SS == 1 else _ss_dissolve(k, W, H)
    else:
        img = render(float(k), W * SS, H * SS)
        if SS > 1:
            img = img.reshape(H, SS, W, SS, 3).mean((1, 3))
    return np.clip(img, 0, 255).astype(np.uint8).tobytes()

def _ss_dissolve(k, W, H):
    set_light(float(k))
    a = smooth(F_HAND - DISSOLVE, F_HAND, float(k)) ** 0.6
    mine = render_new(float(k), W * SS, H * SS).reshape(H, SS, W, SS, 3).mean((1, 3))
    if a <= 0 or audience() is None:
        return mine
    return mine * (1 - a) + render_theirs(float(k), W, H) * a

def blimp_json(path):
    """The airship for the stadium's own scene: its model, where it hangs, its screen and its lights, all in the
    audience design's frame (X right, Y to the stage, Z up; t their show clock)."""
    pts, kinds = BLIMP_LIGHTS
    m2a = lambda v: [float(v[0]) * BSC, float(v[2]) * BSC, float(v[1]) * BSC]   # model (x, y up, z) -> theirs
    d = dict(
        model=_work("models", "blimp.obj"),
        model_built_by="prototypes/models/build_night_models.py blimp()",
        model_axes="the film's: x along the hull, y up, z across (the screen on its -z side); theirs = (x, z, y) * scale",
        scale=BSC,
        centre_at_t0=[float(v) for v in BLIMP_AU], drift_x_per_s=BLIMP_AU_V, bob="Z += 1.2 sin(0.6 t)",
        screen=dict(centre=m2a(BS["c"]), width=BS["w"] * BSC, height=BS["h"] * BSC, tilt_down_rad=BS["tilt"],
                    faces="-Y, tilted down", shows="DEMOSPLASH 0x10 over a laser grid, cycling with demo clips"),
        running_lights=[dict(p=m2a(q), kind=["warm", "frame", "beacon"][int(k)]) for q, k in zip(pts, kinds)],
        belly_searchlights=[m2a(np.array([lx, -17.0, lz])) for lx, lz in ((-30, -6), (-10, 5), (12, -4), (32, 6))],
        uplights=[dict(source=list(src), phase=ph,
                       sweep="the spot on the hull at u = 0.85 sin(0.55 t + phase) along it, v = -pi/2 + 1.25 sin(0.4 t "
                             "+ 1.7 phase) round it (blimp_hull(u, v) in train_night.py), t in seconds")
                  for src, ph in UPLIGHTS],
        colours=dict(hull=[0.055, 0.055, 0.065], frame=[1.0, 0.28, 0.06], warm=[3.0, 1.7, 0.7], beacon=[4.0, 0.2, 0.1]))
    json.dump(d, open(path, "w"), indent=1)

def main():
    args = sys.argv[1:]
    if args and args[0] == "--blimp-json":
        blimp_json(args[1]); return
    def opt(name, default):
        if name in args:
            i = args.index(name); v = args[i + 1]; del args[i:i + 2]; return v
        return default
    W, H = map(int, opt("--size", "1536x768").split("x"))
    jobs = int(opt("--jobs", "10"))
    stills = opt("--stills", None)
    rng_ = opt("--frames", None)
    print("stop at %.0f (frame %d), board %.0f, platforms %.0f-%.0f; chase %d frames, hand-off frame %d at %s; "
          "%d frames in all (the film's: the orbit's %d skipped from %d)" % (ZS, F_STOP, BOARD_Z, PL0, PL1, NCH, F_HAND, (HANDOFF + [BX, 0, 0]).round(1), NFR_FILM, CUT_N, VOL_OFF),
          flush=True)
    from multiprocessing import Pool
    if stills:
        from PIL import Image
        outdir = args[0]
        os.makedirs(outdir, exist_ok=True)
        ks = [float(k) for k in stills.split(",")]
        with Pool(min(jobs, len(ks))) as pool:
            for k, b in zip(ks, pool.imap(frame_bytes, [(k, W, H) for k in ks])):
                Image.fromarray(np.frombuffer(b, np.uint8).reshape(H, W, 3)).save(f"{outdir}/n{k:06.1f}.png")
        return
    out = args[0]
    a, b = (0, NFR_FILM) if not rng_ else map(int, rng_.split(":"))
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}",
                           "-r", str(T.FPS), "-i", "-", "-c:v", "libx264", "-profile:v", "main", "-level", "4.0",
                           "-bf", "0", "-g", "12", "-crf", "17", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out],
                          stdin=subprocess.PIPE)
    with Pool(jobs) as pool:
        for i, bts in enumerate(pool.imap(frame_bytes, [(k, W, H) for k in range(a, b)])):
            ff.stdin.write(bts)
            if i % 24 == 0:
                print("frame", a + i, flush=True)
    ff.stdin.close(); ff.wait()
    print(b - a, "frames,", round((b - a) / T.FPS, 1), "s")

if __name__ == "__main__":
    main()
