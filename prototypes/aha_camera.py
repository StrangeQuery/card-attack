"""The a-ha orbit's camera, measured: the frame's corners and edges read off seven keyframes of the video (1:36-1:43,
every 14th frame at 12 fps, 960x720), and a camera fitted to them by least squares, with the lens, the frame's
proportions and its thickness fitted too. Units: the opening is 1 wide.

    python aha_camera.py            writes aha_camera.json and prints the fit and its residuals

Frame model: the opening is centred on the origin in the plane z = 0 (its front face, the side the camera starts on is
z < 0); the board's border is b wide all round and t thick (back face z = t). Camera k: at angle a round the frame
(0 straight in front, negative to the left), distance d from the frame's centre in plan, height h; looking along yaw
psi (0 = +z), pitch theta, roll rho.
"""
import json
import math
import pathlib

import numpy as np

HERE = pathlib.Path(__file__).resolve().parent
CX, CY = 480.0, 360.0

def corners(H, b, t):
    hw, hh = 0.5, H / 2
    ow, oh = hw + b, hh + b
    P = {}
    for z, pre in ((0.0, ""), (t, "back_")):
        P[pre + "iTL"], P[pre + "iTR"] = (-hw, hh, z), (hw, hh, z)
        P[pre + "iBL"], P[pre + "iBR"] = (-hw, -hh, z), (hw, -hh, z)
        P[pre + "oTL"], P[pre + "oTR"] = (-ow, oh, z), (ow, oh, z)
        P[pre + "oBL"], P[pre + "oBR"] = (-ow, -oh, z), (ow, -oh, z)
    return {k: np.array(v) for k, v in P.items()}

# observations per keyframe: ("pt", corner, (x, y)) or ("line", corner, corner, (x, y)) -- the image point lies on the
# line through the two corners' images. Read off the gridded keyframes (yt-ahaa/kf), about +-8 px.
OBS = {
    0: [("pt", "iTL", (148, 65)), ("pt", "iTR", (813, 105)), ("pt", "iBR", (776, 690)),
        ("line", "iTL", "iBL", (186, 720)), ("pt", "oTR", (862, 62)),
        ("line", "oTL", "oBL", (78, 0)), ("line", "oTL", "oBL", (122, 720)),
        ("line", "oTR", "oBR", (818, 720)), ("line", "oTL", "oTR", (330, 5))],
    14: [("pt", "iTL", (232, 63)), ("pt", "iTR", (810, 110)), ("line", "iTL", "iBL", (255, 720)),
         ("line", "iTR", "iBR", (782, 600)), ("line", "oTL", "oBL", (163, 0)), ("line", "oTL", "oBL", (195, 720)),
         ("pt", "oTR", (853, 72)), ("line", "oTL", "oTR", (330, 0)), ("line", "oTR", "oBR", (822, 700))],
    28: [("line", "back_oTL", "back_oBL", (228, 0)), ("line", "back_oTL", "back_oBL", (255, 720)),
         ("line", "oTL", "oBL", (248, 0)), ("line", "oTL", "oBL", (275, 720)),
         ("pt", "iTL", (307, 55)), ("line", "iTL", "iBL", (322, 720)), ("pt", "iTR", (748, 120)),
         ("pt", "oTR", (782, 85)), ("line", "oTL", "oTR", (420, 0))],
    42: [("line", "back_oTL", "back_oBL", (282, 0)), ("line", "back_oTL", "back_oBL", (292, 720)),
         ("line", "oTL", "oBL", (322, 0)), ("line", "oTL", "oBL", (333, 720)),
         ("pt", "iTL", (357, 30)), ("line", "iTL", "iBL", (370, 700)), ("pt", "iTR", (590, 110)),
         ("line", "iTR", "iBR", (578, 650)), ("pt", "oTR", (608, 80)), ("line", "oTL", "oTR", (490, 0))],
    56: [("line", "back_oTL", "back_oBL", (405, 0)), ("line", "back_oTL", "back_oBL", (405, 720)),
         ("line", "oTL", "oBL", (458, 0)), ("line", "oTL", "oBL", (458, 720)),
         ("line", "back_iTL", "back_iBL", (385, 100)), ("line", "back_iTL", "back_iBL", (395, 620))],
    70: [("line", "back_oTL", "back_oBL", (683, 0)), ("line", "back_oTL", "back_oBL", (640, 720)),
         ("line", "oTL", "oBL", (718, 0)), ("line", "oTL", "oBL", (690, 720)),
         ("line", "back_iTL", "back_iBL", (606, 0)), ("line", "back_iTL", "back_iBL", (590, 720)),
         ("pt", "back_iTR", (240, 75)), ("line", "back_iTL", "back_iTR", (470, 0)),
         ("line", "back_iTR", "back_iBR", (240, 700)), ("line", "back_oTR", "back_oBR", (208, 35)),
         ("line", "back_oTR", "back_oBR", (210, 700))],
    84: [("line", "back_oTL", "back_oBL", (850, 0)), ("line", "back_oTL", "back_oBL", (815, 720)),
         ("line", "oTL", "oBL", (890, 0)), ("line", "oTL", "oBL", (835, 720)),
         ("line", "back_iTL", "back_iBL", (775, 0)), ("line", "back_iTL", "back_iBL", (725, 720)),
         ("pt", "back_iTR", (130, 60)), ("line", "back_iTL", "back_iTR", (500, 0)),
         ("line", "back_iTR", "back_iBR", (148, 700)), ("line", "back_oTR", "back_oBR", (88, 10)),
         ("line", "back_oTR", "back_oBR", (100, 700))],
}
KEYS = sorted(OBS)

def rotation(psi, theta, rho):
    fw = np.array([math.sin(psi) * math.cos(theta), math.sin(theta), math.cos(psi) * math.cos(theta)])
    r = np.cross([0, 1.0, 0], fw); r /= np.linalg.norm(r)
    u = np.cross(fw, r)
    c, s = math.cos(rho), math.sin(rho)
    return np.array([c * r + s * u, -s * r + c * u, fw])

def cam_pos(a, d, h):
    return np.array([d * math.sin(a), h, -d * math.cos(a)])

def project(P, C, R, f):
    q = R @ (P - C)
    return np.array([CX + f * q[0] / q[2], CY - f * q[1] / q[2]])

def unpack(x):
    f, H, b, t = x[:4]
    cams = x[4:].reshape(len(KEYS), 6)
    return f, H, b, t, cams

def residuals(x):
    f, H, b, t, cams = unpack(x)
    P = corners(H, b, t)
    res = []
    for k, (a, d, h, psi, theta, rho) in zip(KEYS, cams):
        C, R = cam_pos(a, d, h), rotation(psi, theta, rho)
        for o in OBS[k]:
            if o[0] == "pt":
                res.extend(project(P[o[1]], C, R, f) - np.array(o[2]))
            else:
                p, q = project(P[o[1]], C, R, f), project(P[o[2]], C, R, f)
                n = np.array([q[1] - p[1], p[0] - q[0]]); n /= np.linalg.norm(n) + 1e-12
                res.append(n @ (np.array(o[3]) - p))
    # weak priors so the unobservable stays sensible: no roll to speak of, a plausible frame
    res.extend(cams[:, 5] * 200)
    res.extend((cams[:, 2] - np.median(cams[:, 2])) * 30)   # the height drifts little (edge-on, nothing else fixes it)
    res.append((t - 0.08) * 50)
    return np.array(res)

def fit():
    a0 = np.radians([-20, -35, -55, -75, -92, -130, -160])
    cams0 = []
    for a in a0:
        d, h = 2.4, 0.15
        C = cam_pos(a, d, h)
        psi = math.atan2(-C[0], -C[2])
        cams0.append([a, d, h, psi, -0.05, 0.0])
    x = np.concatenate([[880, 1.2, 0.12, 0.08], np.array(cams0).ravel()])
    lam = 1e-3
    r = residuals(x)
    for it in range(300):
        J = np.zeros((len(r), len(x)))
        for j in range(len(x)):
            dx = np.zeros_like(x); dx[j] = 1e-5 * max(1.0, abs(x[j]))
            J[:, j] = (residuals(x + dx) - r) / dx[j]
        A, g = J.T @ J, J.T @ r
        step = np.linalg.solve(A + lam * np.diag(np.diag(A) + 1e-9), -g)
        r2 = residuals(x + step)
        if r2 @ r2 < r @ r:
            x, r, lam = x + step, r2, lam * 0.3
            if np.abs(step).max() < 1e-7:
                break
        else:
            lam *= 10
    return x, r

def main():
    x, r = fit()
    f, H, b, t, cams = unpack(x)
    print(f"lens f = {f:.0f} px (vertical field {math.degrees(2 * math.atan(CY / f)):.1f} deg at 4:3); "
          f"opening 1 x {H:.3f}; border {b:.3f}; thickness {t:.3f}")
    print("frame   angle    dist   height    yaw    pitch   roll")
    for k, c in zip(KEYS, cams):
        print(f"{k:5d} {math.degrees(c[0]):8.1f} {c[1]:7.2f} {c[2]:7.2f} {math.degrees(c[3]):7.1f} {math.degrees(c[4]):7.1f} {math.degrees(c[5]):6.1f}")
    per = r[:-(2 * len(KEYS) + 1)]
    # the worst observations, to catch misreadings
    f_, H_, b_, t_, cams_ = unpack(x)
    P = corners(H_, b_, t_)
    worst = []
    for k, c in zip(KEYS, cams_):
        C, R = cam_pos(*c[:3]), rotation(*c[3:])
        for o in OBS[k]:
            if o[0] == "pt":
                e = np.linalg.norm(project(P[o[1]], C, R, f_) - np.array(o[2]))
            else:
                p, q = project(P[o[1]], C, R, f_), project(P[o[2]], C, R, f_)
                n = np.array([q[1] - p[1], p[0] - q[0]]); n /= np.linalg.norm(n)
                e = abs(n @ (np.array(o[-1]) - p))
            worst.append((e, k, o))
    for e, k, o in sorted(worst, key=lambda w: -w[0])[:6]:
        print(f"  {e:5.1f} px  frame {k}: {o}")
    print(f"residuals: rms {np.sqrt((per ** 2).mean()):.1f} px, worst {np.abs(per).max():.1f} px over {len(per)}")
    out = {"f": f, "H": H, "border": b, "thickness": t, "keys": KEYS,
           "cams": [dict(zip(("a", "d", "h", "yaw", "pitch", "roll"), map(float, c))) for c in cams]}
    (HERE / "aha_camera.json").write_text(json.dumps(out, indent=1))

if __name__ == "__main__":
    main()
