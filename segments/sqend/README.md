# SQ (the ending)

The film's last segment: a big SQ in silver chrome, with STRANGE QUERY coming in under it.

![The SQ in silver chrome, STRANGE QUERY under it](screenshot.png)

## What it shows

After the QR code, a hard cut (a stutter doesn't work at 12 fps) to a big SQ in silver chrome,
no colour, like THX's. The letters are heavy and packed tight after HBO's wordmark: the S butts into the Q, the Q's
counter is open, and its tail is cut flat on the baseline. They are extruded and seen in close from the ground, low
and to the left, so their bottom is one straight line and the near end looms. Faint stars are behind them. From
frame 8, STRANGE QUERY, spaced out and chrome too, materialises under the letters over 14 frames: each of its pixels
at its turn is a white sparkle for two frames, then its chrome. 36 frames at 1536x768, the film's last 3 seconds
(`seg sqend sqend/video_hd.rgb 0 36 1536x768` in the cut). The design is `prototypes/ending_sq.py`.

## How it's done

The SQ is an asset: a render made ahead of time in Python (`data/make_data.py` makes `data/sqend.bin`, kept packed
in `generated.tar.xz`). The letters' shapes, where each pixel's reflection points and its shading are worked out
there, not on CARDIAC. For each pixel it holds T, where the pixel's reflection goes (in thousandths, from straight up
to straight down), a multiplier M and an addition A; for the name it holds each pixel's cover, its own T and its
turn to come in (the sparkle's order). CARDIAC lights it from the surround: a grey for each pixel worked out by
comparisons at its T, times M, plus A, and it brings the name in frame by frame.

The renderer is [`sqend.rs`](sqend.rs), written once against the train's operations (`segments/train/ops.rs`). The
host runs it on the train's machines (`host/src/fx/sqend.rs`, through `fx::train::Cardiac`), and `reference.rs`
runs the same source natively.

- **Once, on CARDIAC, as the segment's first job during the film:** the surround's grey for each of the 375,792
  chrome pixels at its T. The surround is
  piecewise straight: 60 high up to 245 at the horizon, a dark band from 20 to 35, then 35 to 120 and 120 to 170
  below. Each grey is chosen by comparisons, times M over 1000, plus A, held to 0..255. The name's 10,042 pixels get
  their chrome the same way.
- **Each frame:** how far the name has come in, which of its pixels are in and which still sparkle (a pixel sparkles
  for 143 thousandths of the way after its turn), each mixed over the picture by its cover. The host copies the
  picture's bytes and fetches by the indices the machines punched.

36 frames, 24,000 machine runs, 1,337M instructions.

## Files

- `sqend.rs`: the renderer, shared by the host and the reference.
- `data/make_data.py`: makes `data/sqend.bin` (packed in `generated.tar.xz`; `build.sh` unpacks it).
- `reference.rs`: draws every frame natively from the same source. `card-attack sqend refs/sqend_ref` checks every
  frame against it and says identical 36/36.
- Host: [`host/src/fx/sqend.rs`](../../host/src/fx/sqend.rs). It files `video_hd.rgb` (not kept).
