"""The records the host reads from a part's data/*.bin (the train's, the night's, the rooms', the SQ ending's): each a
name, a shape and its numbers, whole numbers stored in the fewest bytes that hold them all (1, 2 or 4).

A record: u32 the name's length, the name, u32 the shape's count of dimensions with the bytes per number in its
second byte (0 for 4, as older files were written), u32 each dimension, then the numbers, little-endian.
The narrow widths keep the demo inside the party's 1 GB."""
import struct

import numpy as np


def width(a):
    """The fewest bytes (1, 2 or 4) that hold every number of A."""
    lo, hi = (int(a.min()), int(a.max())) if a.size else (0, 0)
    assert -2 ** 31 <= lo and hi < 2 ** 31
    return 1 if -128 <= lo and hi < 128 else 2 if -32768 <= lo and hi < 32768 else 4


def record(name, a, shape=None):
    """One record's bytes: NAME, A's numbers, and SHAPE (A's own if not given)."""
    a = np.asarray(a)
    if a.dtype.kind == "b":
        a = a.astype(np.int64)
    shape = a.shape if shape is None else tuple(shape)
    w = width(a)
    nb = name.encode()
    head = struct.pack("<I", len(nb)) + nb + struct.pack("<I", len(shape) | w << 8) + struct.pack(f"<{len(shape)}I", *shape)
    return head + np.ascontiguousarray(a, dtype=f"<i{w}").tobytes()
