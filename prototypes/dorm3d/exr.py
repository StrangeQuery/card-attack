"""Reading room.py's and hall.py's multilayer EXRs: {pass name: float array (h, w, c)}."""
import numpy as np
import OpenEXR


def read(path):
    f = OpenEXR.File(path)
    out = {}
    for p in f.parts:
        for name, ch in p.channels.items():
            out[name.split(".", 1)[1] if "." in name else name] = np.asarray(ch.pixels, np.float32)
    return out
