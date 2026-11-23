// The machine's view of the world, included by every machine written in Rust. rust-cardiac (cargo cardiac build) turns
// calls to these into instructions, as it does for its own built-ins crate, `cardiac`: it knows them by name. Built
// with `--cfg native`, they have bodies that read and punch cards on the host instead (cardiac_host::native), so the
// same machine runs as an ordinary Rust function on the deck the host deals it. (The compiler used to ship this file;
// its built-ins are now the `cardiac` crate, whose native side is standard input and output, which is no use to a
// host dealing decks, so Card Attack keeps this one.)

/// The card reader: the next card in the deck. INP.
#[cfg(not(native))]
#[inline(never)]
pub fn inp() -> i16 { 0 }

/// The card punch. OUT.
#[cfg(not(native))]
#[inline(never)]
pub fn out(_v: i16) {}

/// The end of a phase, for a program too big for memory: the loader reads the next phase's code in over this
/// phase's (the data stays). JMP 00.
#[cfg(not(native))]
#[inline(never)]
pub fn overlay() {}

/// What the deck guarantees (a card's range, say), so the compiler can leave out the checks it would otherwise need.
/// No code: it is core::hint::assert_unchecked. With debug assertions on, it is checked, an assert!.
///
/// # Safety
///
/// `condition` must hold: without debug assertions a false assumption is undefined behaviour.
#[cfg(not(native))]
#[allow(dead_code)]
#[inline(always)]
pub unsafe fn assume(condition: bool) {
    if cfg!(debug_assertions) {
        assert!(condition);
    } else {
        // SAFETY: the caller's guarantee that condition holds
        unsafe { core::hint::assert_unchecked(condition) }
    }
}

/// A table the host punches into the program deck (as hand-written machines carry a table at a label): the array
/// a `let` binds to this lives at the label resource_NAME (resource_map for `let map = load_resource("map");`). Only
/// in main, outside any loop, since the loader fills it once. No code.
#[cfg(not(native))]
#[inline(never)]
pub fn load_resource<const N: usize>(_name: &str) -> [i16; N] { [0; N] }

/// Digit k of a card's three (k = 0 the hundreds, 1 the tens, 2 the units), for a value 0..999. On CARDIAC: the
/// shift that keeps that digit, SFT (k+1)3, built for k and run, as hand-written machines pick a texel out of a
/// cell. Built in: the body here is what it means (and what a native build runs).
#[inline(never)]
pub fn digit(v: i16, k: i16) -> i16 { if k == 0 { v / 100 } else if k == 1 { v / 10 % 10 } else { v % 10 } }

// The same built-ins, built natively: the deck and the punched cards are the host's (see cardiac_host::native).

/// The next card of the deck the host handed in.
#[cfg(native)]
#[inline]
pub fn inp() -> i16 { ::cardiac_host::native::inp() }

/// Punch a card; in a debug build, a value outside -999..999 is an error.
#[cfg(native)]
#[inline]
pub fn out(v: i16) { ::cardiac_host::native::out(v) }

/// Nothing: natively every phase runs in turn, in one function.
#[cfg(native)]
#[inline]
pub fn overlay() {}

/// Checked: a deck that breaks an assumption is an error here, where on CARDIAC it would be undefined.
///
/// # Safety
///
/// `condition` must hold: on CARDIAC a false assumption is undefined behaviour.
#[cfg(native)]
#[allow(dead_code)]
#[track_caller]
pub unsafe fn assume(condition: bool) {
    assert!(condition, "the deck broke an assumption");
}

/// The table the host gave for this run.
#[cfg(native)]
#[inline]
pub fn load_resource<const N: usize>(name: &str) -> [i16; N] { ::cardiac_host::native::load_resource::<N>(name) }
