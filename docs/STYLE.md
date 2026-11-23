# Writing a machine

A machine is a Rust program in a segment's `machines/` folder that rust-cardiac compiles to a CARDIAC tape. It has to fit in 96 cells,
and a person should be able to read it. Each machine is a binary target of the machines' package, `segments/Cargo.toml`:

    [[bin]]
    name = "NAME"
    path = "SEGMENT/machines/NAME.rs"

## Shape

    //! Name, once per <unit> per frame of <segment>. Deck: what it reads, in order. What it does, in a sentence or
    //! two. Punches: what it punches, in order.
    // cardiac: replaces SEGMENT/original/FILE.asm
    include!("../../shared/cardiac.rs");
    include!("../../shared/prelude.rs");

    pub fn main() {
        ...
    }

The header comment is the machine's contract: its cards in, its cards out. Keep it exact.

## Readable

- **Names say what a value is:** `threshold`, `fade`, `row_start`, not `t`, `k`, `n`. `x` and `y` for screen
  coordinates and `i` for a plain index are fine.
- **Use the prelude.** It brings in demokit, a crate the package depends on:
  - `Pixel`, with `read_pixel()` and `punch(p)` for colours;
  - `tables::*` for lookup tables, never a copied list of numbers;
  - `lcg::next` for random numbers;
  - `isqrt`, `abs`, `abs_diff`, `wrap`, `remainder` and `divide` for arithmetic by adding and subtracting.
- **Name a step with a small function** when it helps. At opt-level 2 rustc inlines it, so it costs nothing.
  Mark one `#[inline(never)]` only when it is called from several places and a single copy is smaller.
- **Comment what CARDIAC forces.** Code bent by the machine should say why, the first time each form appears in a
  file:
  - division by repeated subtraction;
  - squares by adding;
  - a loop that counts up to zero because `c >= 0` is the cheapest test;
  - an `assume`, with a `// SAFETY:` comment saying where its guarantee comes from.

  Don't comment ordinary Rust.
- **Comments say what the code does now.** No history, dates, names or issue numbers: that's what commit
  messages are for. Keep comments light and plain, with no jargon.
- **One statement per line,** except for very short paired ones (`a += 1; if a >= 16 { a -= 16; }`).

## What the machines use

rust-cardiac compiles much more of Rust than these machines use (iterator adaptors, closures, recursion and more:
its [reference guide](https://lab.strangequery.com/rust-cardiac/manual/) has the list). The machines keep to the
plain forms below, which fit 96 cells:

- **Types:** `i16` (a card, -999..999), `u8`, `bool`, `usize` for indexes. Arrays of those, and structs of those
  (`Pixel`).
- **Loops:** `for` over a range (`0..n`, `0..=n`, either one `.rev()`), `while` and `loop`. A range loop costs
  the same as the `while` it stands for.
- **No** floats or heap: CARDIAC has neither.
- **demokit's functions and consts** compile like the machine's own.
- **Deck guarantees:** `unsafe { assume(cond) }` states what the deck guarantees (a clock is 0..15), so bounds
  checks can go. It costs nothing, and it must be true: say why in a `// SAFETY:` comment above it. Several in a
  row share one `unsafe` block.
- **Tables:** `load_resource("NAME")` for a table the host punches into the program (in main, before any loop); `digit(v, k)` for digit k of a card.

## Checking

    cargo cardiac build --manifest-path segments/Cargo.toml --bin NAME    # prints its size: 96 cells or fewer
    CARD_ATTACK_SHADOW=1 host/target/release/card-attack SEGMENT refs/SEGMENT_ref
    host/target/release/card-attack --native EFFECT refs/EFFECT_ref

Every frame must still be identical. `CARD_ATTACK_SHADOW=1` also runs the hand-written original on the same
cards for one-phase machines. `--native` also runs the machine built natively from the same source, and every
card it punches must match CARDIAC's. A debug build of the host (`cargo build`, then `host/target/debug/card-attack`)
also stops at any native punch outside -999..999.
