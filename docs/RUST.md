# Card Attack in Rust

Every machine the film runs was first written in Rust and compiled to a CARDIAC tape. There are 443 machine
programs in `segments/*/machines/` (about 8,100 lines, not counting comments, blank lines and the two `include!`
lines each starts with), each a binary target of one Cargo package, `segments/Cargo.toml`. The film runs 445
machines, 20 of them written by hand in CARDIAC assembly for speed (below), and `card-attack play --dry-run`
counts 2,641.7G CARDIAC instructions on them for the cut's 3,419 frames (4:44.9 at 12 frames a second). Each segment
and the cut are checked frame by frame against an independent native Rust reference, and all match.

## The pipeline

    machines/NAME.rs + demokit --rustc, run as the CARDIAC driver (nightly-2026-05-22)--> MIR --rust-cardiac's backend--> CARDIAC tape

- **rustc** does the front end: parsing, types, the borrow checker and its own optimisation, at opt-level 2.
- **rust-cardiac** (github.com/StrangeQuery/rust-cardiac, installed by following its README, 0.2.5 or later) is
  `cargo cardiac` and its driver: rustc built as a library, on the one nightly it pins, with a backend that
  turns each machine's MIR into CARDIAC code and writes its tape (`NAME.cqb`, and `NAME.2.cqb` ... for a machine in
  phases), its panic map (`NAME.map`) and its resources' slots (`NAME.cqb.meta`).
- **Cargo** says what the machines depend on: demokit (`demokit/`, the tables and the arithmetic), whose MIR the
  driver compiles along with whatever part of it a machine reaches. The target, `cardiac-unknown-none`, isn't one
  rustc knows, so `cargo build` doesn't build machines; `cargo cardiac build --manifest-path segments/Cargo.toml`
  does (`build.sh` runs it). The package's dev profile is opt-level 2 with overflow checks, debug assertions and
  debuginfo off.
- **The host** (`host/`, Rust) is the sequencer. It keeps the schedule (which frame of which segment comes next),
  deals cards in, collects the cards punched out, fetches from tables at the places machines punch, and lays the
  results into frames.
- **Machines** are ordinary CARDIACs: 100 cells, the original ten instructions, booted from their tapes. A machine
  too big for memory runs in phases (`overlay()`): each phase loads its code over the last one through cell 00, and
  the data stays put. 17 of the 443 are in phases.

## What is written by hand

The film's hottest machines are written in CARDIAC assembly, in the `hand/` folders of the train and the swarm:
20 machines, and between them 1,991.2G of the 2,641.7G instructions (75%). Seventeen are the same machine as a Rust
one, card for card: the host loads `PART/hand/NAME.asm` in place of the compiled `NAME.cqb`, and
the Rust machine's native build runs beside it in the native checks. The other three (`tr_add1`, `tr_add2p`,
`tr_add2m`) each do one case of adding longs. The host picks the case by working out the sums itself first, from
the actual values, and then deals the cards to that case's machine. [HOT_MACHINES.md](HOT_MACHINES.md) has how each was made and what it saves.

The rest of the film, 649.0G instructions on 425 machines, runs compiled Rust.

There are also seven hand-written originals in `original/` folders (the cube's `project`, `esort`, `gather`, `fshade`
and `palette`, the cut's dissolve and the rotozoomer's paint). A Rust machine that starts with the line `// cardiac:
replaces PART/original/FILE.asm` stands in for that file wherever the host asks for it, dealt the same cards, and
`CARD_ATTACK_SHADOW=1` runs the original beside it and stops at the first card that differs.

Code that no Rust source line produced:

- **Runtime routines the compiler emits** when a program needs them: multiplying and dividing by a variable (by
  repeated adding and taking away; by a constant it is inlined as adds and decimal shifts), and the panic, which
  punches -999 and halts.
- **Built-ins.** Each is a Rust function in `segments/shared/cardiac.rs` (Card Attack's copy of rust-cardiac's
  built-ins, whose native side reads and punches the host's cards), with a body that says what it means and is what
  a native build runs. The compiler turns each call straight into CARDIAC code:

  | Built-in | Becomes |
  |---|---|
  | `inp()` | INP |
  | `out(v)` | OUT |
  | `overlay()` | the end of a phase: JMP 00, the loader reads the next phase |
  | `load_resource("NAME")` | a table the host punches into the program deck at a named label (`resource_NAME`); only in main, outside any loop |
  | `digit(v, k)` | digit k of a three-digit card: a shift instruction built for k and run |
  | `unsafe { assume(c) }` | nothing |

  - `assume(c)` is `core::hint::assert_unchecked`, an `unsafe fn` as in Rust. It costs no code, and it tells the
    compiler what the deck guarantees (that a clock card is 0..15, say), so bounds checks can go. A false
    assumption is undefined behaviour, so the call says `unsafe`, with a `// SAFETY:` comment naming where the
    guarantee comes from. Natively it is checked; with debug assertions on, it is checked on CARDIAC too.
  - `digit` is where the compiler writes in a hand-written trick for something Rust has no compact form for: the
    shift loop it stands for costs about 9 cells.

## Cards versus Rust's types

- **An i16 is one card, -999..999.** On CARDIAC a cell holds a sign and three digits; the accumulator holds a sign
  and four. Past ±999, a value loses digits when stored.
- **Checks are compiled like a release build.** Overflow checks are off; bounds checks stay. The range analysis
  removes the bounds checks it can prove, and `assume` covers what only the deck knows.
- **Every frame is verified,** so a value that went wrong would show. `card-attack --native ...` also runs every
  Rust machine built natively on the same cards and stops at the first card that differs from CARDIAC's.

## Tools

- **Rebuild:** `./build.sh` builds every machine with the installed `cargo cardiac`; one machine:
  `cargo cardiac build --manifest-path segments/Cargo.toml --bin NAME`.
  `--emit deck,listing,map` writes each machine's listing (`NAME.cq`) beside its tape.
- **Tests:** the compiler's own tests, which compare each program's output on CARDIAC with native Rust, are in
  rust-cardiac's repository (its `CONTRIBUTING.md` says how to run them).
- **Render:** `./refs.sh` builds the references; then `host/target/release/card-attack all refs` renders and checks
  every segment and the cut. Set `PROFILE=file` to count instructions per machine.
- **Native check:** `host/target/release/card-attack --native all refs` also runs every machine built natively from
  its Rust source (the built-ins in `segments/shared/cardiac.rs` working on the host's cards) and stops at any card
  that differs from CARDIAC's; `CARD_ATTACK_NATIVE=only` runs them natively in CARDIAC's place.
