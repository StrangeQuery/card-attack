"""The printed sine table for the ray's script: 999 sin of k 120ths of a turn, k 0..119, rounded. The script
machine (rt_script) makes the ball's sway from it.

    python3 segments/ray/data/make_sine.py > segments/ray/data/sine120.txt
"""
import math

print("# k (120ths of a turn): 999 sin")
for k in range(120):
    print(math.floor(999 * math.sin(2 * math.pi * k / 120) + 0.5))
