# HEARTBURN against Python: blasting pixels

How fast does an emulated cardboard computer put pixels on the screen, next to the languages people prototype in?
Measured 2026-10-25 on the plasma, the cleanest per-pixel effect in the film.

## The test

The [plasma](../segments/plasma) at 192x96, 204 frames: four waves from small tables and the distance from a centre
(an integer square root, counted out), summed into a place on an eight-colour ring. The same formula
([`formula.rs`](../segments/plasma/formula.rs)) four ways, each on one core:

- **HEARTBURN:** the plasma's CARDIAC machines, as the film runs them (`card-attack plasma refs/plasma_ref`). Every
  frame is about 22 million CARDIAC instructions: three-digit decimal cards, add and subtract, no multiply.
- **Native Rust:** the plasma's reference (`refs/plasma_ref segments/plasma/data/ring.txt
  segments/plasma/data/script.txt`), built with `rustc -O`, its frames written to /dev/null.
- **Plain Python:** [`bench/plasma_py.py`](bench/plasma_py.py), the formula ported line for line (the same wave
  tables, the same square root counted in a loop), a loop over the pixels.
- **NumPy:** the same script's second half: the whole frame at once in array operations. It takes the square root in
  floating point, where every other version counts it out, so it is not quite the same work.

Apple M5 Ultra, rustc 1.97.1, Python 3.9.6, NumPy 2.0.2.

## The numbers

| How | Square root | ms a frame | Pixels a second | Against HEARTBURN |
|---|---|---|---|---|
| Rust, tables built once | floating point | 0.054 | 340M | 460x faster |
| NumPy, the frame at once | floating point | 0.48 | 38M | 52x faster |
| Rust, tables built once | counted | 1.11 | 17M | 23x faster |
| Rust, the reference as written | counted | 2.4 | 7.7M | 10x faster |
| **HEARTBURN (CARDIAC machines)** | **counted** | **25** | **740k** | |
| Plain Python, a loop over pixels | counted | 201 | 92k | 8x slower |

The two "tables built once" rows are [`bench/plasma_fast.rs`](bench/plasma_fast.rs) (`rustc -O`, then
`plasma_fast segments/plasma/data/ring.txt exact` or `float`): the same formula, the frame in a buffer, nothing written
out.

HEARTBURN ran 4,553M CARDIAC instructions in 5.11 s here: about 0.9G instructions a second on one core.

## What it says

- **The cardboard computer beats Python by 8x.** Python runs the same formula, with real multiplication and real
  integers, and is still eight times slower than HEARTBURN running CARDIAC machines that multiply by adding.
- **HEARTBURN is about 23x behind Rust doing the same work** (the square root counted, no waste), and 10x behind the
  reference as written: an interpreter of a 1968 teaching computer, every multiply an add, in three-digit decimal.
- **Why NumPy first looked faster than Rust:** it wasn't doing the same work. The reference is written to be plainly
  right, not quick: its wave lookup builds a fresh `Vec` from the table every call (four a pixel, some 74,000 heap
  allocations a frame), and with its output that is about half its time. The rest is the square root, counted in a
  loop as the machines must (hundreds of steps a pixel at the frame's edges). NumPy takes the square root in floating
  point, one hardware instruction for several pixels at once, and runs a couple of dozen whole-array operations in
  compiled C, so Python's own slowness is paid per operation, not per pixel. Given the same shortcut, Rust is 9x
  faster than NumPy; the counted square root alone is a factor of 20.
- **That is why a NumPy version can't be a reference:** the floating-point shortcuts are what make it fast, and
  they aren't what a machine does.
- **On this Mac's 28 workers** HEARTBURN puts out about 20 million plasma pixels a second: 192x96 at about 1,000
  frames a second, or 1536x768 at about 17, for pixels as costly as the plasma's. The film computes ahead of its
  frames, so the full-resolution segments only need to keep up on average. All of this is during the film: the
  player starts at most one second before frame 0, and a segment's frames may be computed while earlier segments
  are on screen and held in memory until they are due.

One effect, one run each; the reference's time includes writing its frames out. To repeat:

    host/target/release/card-attack plasma refs/plasma_ref
    time refs/plasma_ref segments/plasma/data/ring.txt segments/plasma/data/script.txt > /dev/null
    python3 docs/bench/plasma_py.py          # from the repo's root; needs numpy
    rustc -O -o /tmp/plasma_fast docs/bench/plasma_fast.rs && /tmp/plasma_fast segments/plasma/data/ring.txt exact
