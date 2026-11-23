# Overlays

Machines that dither a caption over a picture. No segment in the film uses them, but the cut's composer
(`host/src/fx/cut.rs`) loads them when it starts, so they are kept here for the build.

- `cwid` works out a caption box's width and `cap` dithers the caption in, in the listing's font. Both exist only as
  hand-written CARDIAC, in `original/`.
