# Shared

Machines and data that more than one segment uses, and the two files every machine includes. This isn't a
segment of its own.

![The VGA segment's sixteen-colour plasma, painted by paint16](screenshot.png)

## What's in it

- **`paint16`** colours the [VGA](../vga) segment's sixteen-colour plasma (the frame above), a half row at a time,
  from the EGA's palette.
- **`paint9`** colours the [rotozoomer](../roto). The host asks for the hand-written `original/paint.asm`, and since
  `paint9.rs` says it replaces that file (`// cardiac: replaces shared/original/paint.asm`), the Rust machine runs.
- **`data/trig.txt`** is read by the [polys](../polys).
- **`prelude.rs` and `cardiac.rs`** are included by every machine written in Rust.
- **`none_data.asm`** is the empty data block the host assembles every one-phase hand-written machine with.
- **`records.py`** writes and reads the records in the `data/*.bin` files of the train, the night, the stadium, the
  rooms and the SQ ending; their generators import it.

## The paint machines

Most segments don't compute red, green and blue directly. They compute a small colour code per pixel, and a paint
machine turns the codes into colours through a palette. This is
[indexed colour](https://en.wikipedia.org/wiki/Indexed_color), the way most 1980s and early 1990s graphics hardware
worked: an index per pixel, and the colours themselves in a separate table. On CARDIAC it also saves memory, since a
colour code is one card and a colour is three.

The sixteen-colour palette alone is 48 cards, half of memory, so `paint16` runs in two phases: the first reads the
palette, and the second loads over the first one's loop and paints the row, 96 codes in and 96 colours out.

## The trig table

`data/trig.txt` is 192 numbers, 99 sin(2π k / 192) for k = 0..191, a printed table. The polys' host takes every
second entry of its first quarter (26 numbers) and punches them into `polysine`'s program as a table; `polysine`
works out the sines from it.

## Machines

| Machine | Cells | What it does |
|---|---|---|
| `paint16` | data 49, code 22 / 31 (two phases) | Per half row: 16 colours in, then each pixel's code to its colour |
| `paint9` | 82 | Per run of 96 pixels: nine colours in, then each pixel's code to its colour, in one phase |

## The prelude

`cardiac.rs` is the machines' view of the world: the card reader `inp()`, the punch `out()` and the other built-ins.
`cargo cardiac build` turns calls to them into instructions; built natively (`--cfg native`), they read and punch
cards on the host instead, so the same machine runs as an ordinary Rust function on the deck the host deals it.
`prelude.rs`, included after it, brings in [demokit](../../demokit) (with `Pixel`, the lookup tables, the arithmetic
and the random numbers) and two small helpers, `read_pixel()` (three cards in) and `punch(p)` (three cards out).
They are small enough that the compiler inlines them, so they cost no cells.

## Files

- `machines/`: `paint16` and `paint9` in Rust.
- `original/paint.asm`: the hand-written version of `paint9` (`CARD_ATTACK_SHADOW=1` runs it beside it).
- `data/trig.txt`: the trig table.
- `cardiac.rs`, `prelude.rs`: included by every machine.
- `none_data.asm`: the empty data block a one-phase hand-written machine is assembled with.
- `records.py`: the format of the generators' `.bin` records.
