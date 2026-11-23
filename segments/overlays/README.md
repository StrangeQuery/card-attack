# Overlays

Machines that dither a big word or a caption over a picture. No segment in the film uses them, but the cut's
composer (`host/src/fx/cut.rs`) loads them when it starts, so they are kept here for the build.

- `big` takes one screen row. Where the word has a letter or its rim and the fade beats that pixel's place in the
  4x4 Bayer map (`BAYER4` in [demokit](../../demokit)), it punches white for a letter or black for the rim;
  elsewhere it passes the picture's pixel through.
- `cwid` works out a caption box's width and `cap` dithers the caption in, in the listing's font. Both exist only as
  hand-written CARDIAC, in `original/`.

The word bitmaps in `words/` are assets made by `make_word.py`, which draws each line in Inter Display Black at 12
pixels cap height, centred on 96x48, with a one-pixel rim round the letters. In each file 1 is a letter, 2 the rim
and 0 nothing.
