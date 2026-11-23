# Listing

Machines that set text in a 3x5 font, a row of pixels at a time. No segment in the film shows them, but the cut's
composer (`host/src/fx/cut.rs`) loads them when it starts, so they are kept here for the build.

- `ltex` reads each character's five font cards for one glyph row of a line and punches that row, a card per
  character.
- `lsel` takes a scroll position and a screen row and punches which line of the text and which of its glyph rows
  that screen row shows.
- `lrow` carries one row of 24 glyph rows in its program (the host punches them in at the label `main_tex`), takes a
  colour code and punches 96 codes: that colour where a glyph pixel is lit, else 0. It picks each pixel with
  `digit()`, which takes one digit out of a three-digit card with a single shift.

`data/font.json` is the font, an asset. `original/` holds the hand-written CARDIAC versions.
