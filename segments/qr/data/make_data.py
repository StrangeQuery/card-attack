# The QR code's modules (segno: python make_data.py): qr.txt, a
# row of 0s and 1s per module row (1 dark), the quiet zone not included. Error correction M, so it scans off a
# projector with some of it lost.
import os
import segno

URL = os.environ.get("CARD_ATTACK_QR_URL") or exit("set CARD_ATTACK_QR_URL to the address the code holds")
here = os.path.dirname(os.path.abspath(__file__))
q = segno.make(URL, error="m", micro=False)
rows = ["".join("1" if m else "0" for m in row) for row in q.matrix]
with open(os.path.join(here, "qr.txt"), "w") as f:
    f.write(f"# the QR code: version {q.version}, error correction {q.error}, {len(rows)} modules a side, "
            "1 dark: see make_data.py\n")
    for r in rows:
        f.write(r + "\n")
