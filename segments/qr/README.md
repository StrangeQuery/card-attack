# QR (before the SQ)

One frame between the stadium and the SQ: a QR code filling the screen, white on black.

![The QR code, white on black, in the middle of the frame](screenshot.png)

## What it shows

The code leads to the blooper reel's hidden page. Its path is an opaque one in capitals, so the code takes the denser
alphanumeric mode and fits version 2. At 12 frames a second it is a flash of white pattern: people know they saw a
QR code and go hunting for it in the video. The film shows it once, as its own segment (`seg qr qr/video_hd.rgb 0 1
1536x768` in the cut).

## How it's done

The picture is an asset: the code's modules, made ahead of time by segno. `qr_line` turns each module into its
pixels. `data/make_data.py` makes `data/qr.txt` with [segno](https://github.com/heuer/segno) (BSD): version 2, error
correction M, 25 modules a side, a row of 0s and 1s per module row (1 dark). It takes the address from the
`CARD_ATTACK_QR_URL` variable, so the address itself is only in the code.

Per module row, the machine `qr_line` colours the row's pixels: white for a dark module, black for a light one, 24
pixels a module. The host (`host/src/fx/qr.rs`) files the frame black and each machine row 24 pixel rows high, so
the code is 600 pixels a side in the middle of the 1536x768 frame and the black around it is the quiet zone.

| Machine | Cells | What it does |
|---|---|---|
| `qr_line` | 33 | Per module row: each module's 24 pixels, white for a dark module and black for a light one |

One frame, 25 machine runs, 0.15M instructions.

The code reads as the URL at 1536x768 and shrunk to 384x192 (OpenCV's QR detector, given it inverted: as it is,
white on black, OpenCV doesn't find it, and phones' readers vary).

## Files

- `machines/qr_line.rs`: the machine.
- `data/qr.txt`: the code's modules; `data/make_data.py` makes it.
- `reference.rs`: draws the same frame in ordinary Rust. `card-attack qr refs/qr_ref` checks it and says identical
  1/1.
- Host: [`host/src/fx/qr.rs`](../../host/src/fx/qr.rs). It files `video_hd.rgb` (not kept).
