"""The printed table of the plasma's tilt: for each thousandth u of the way through the tilt back (0..1000),
the eased angle's 600 cos and 100 sin, rounded as JavaScript does: the angle a quarter turn times 3 s^2 - 2 s^3,
s = u / 1000. The tilt machine (pl_tilt) works out u each frame; the host fetches the line.

    python3 segments/plasma/data/make_tilt.py > segments/plasma/data/tilt.txt
"""
import math

print("# u (thousandths of the tilt): 600 cos, 100 sin of the eased angle")
for u in range(1001):
    s = u / 1000
    a = math.pi / 2 * (3 * s * s - 2 * s * s * s)
    print(math.floor(600 * math.cos(a) + 0.5), math.floor(100 * math.sin(a) + 0.5))
