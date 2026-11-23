#!/usr/bin/env python3
"""A test layer for the cut's "over" line: a notification card, 480x144, as frames of red, green, blue and cover
(0-32, in 32nds), writing layer.rgba (not kept in git). Eight frames: the card's cover rising from 4 to 32 32nds,
its rounded corners soft-edged; the last is held by the cut.

    python3 segments/cut/test/make_layer.py"""
import os

W, H, R, FRAMES = 480, 144, 24, 8
here = os.path.dirname(os.path.abspath(__file__))

def cover(x, y):
    # 32nds of the pixel inside the rounded rectangle, sampled 4 x 4
    inside = 0
    for sy in range(4):
        for sx in range(4):
            px, py = x + (sx + 0.5) / 4, y + (sy + 0.5) / 4
            cx = min(max(px, R), W - R)
            cy = min(max(py, R), H - R)
            inside += (px - cx) ** 2 + (py - cy) ** 2 <= R * R
    return inside * 2

def colour(x, y):
    if x < 12:
        return (88, 101, 242)                       # the accent bar
    if 36 <= x < 108 and 36 <= y < 108:
        return (88 + x % 16 * 8, 160, 220 - y % 16 * 8)   # the icon
    if 132 <= x < 420 and (40 <= y < 56 or (76 <= y < 88 and x < 380) or (100 <= y < 112 and x < 300)):
        return (230, 230, 236)                       # lines of text
    return (43, 45, 49)

out = bytearray()
for f in range(FRAMES):
    for y in range(H):
        for x in range(W):
            r, g, b = colour(x, y)
            out += bytes((r, g, b, cover(x, y) * (f + 1) // FRAMES))
open(os.path.join(here, 'layer.rgba'), 'wb').write(out)
