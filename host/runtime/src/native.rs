//! Native runs: each machine written in Rust also built as an ordinary Rust function (by the demo, which includes
//! every machine's source with `--cfg native`, so that `segments/shared/cardiac.rs` gives the built-ins the bodies
//! below), run on the same deck as on CARDIAC, and its punched cards compared card for card. A direct check of the
//! compiler's code, on top of the reference check.
//!
//! The deck, the punched cards and the carried table live here, per thread, for `run`: the machine's `inp()`,
//! `out()` and `load_resource()` read and punch them. (This was part of the compiler, before rust-cardiac was released on its own:
//! dealing a deck to a machine is the host's business, not the compiler's.)
use std::cell::RefCell;
use std::sync::OnceLock;

/// What the host does with the native builds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Native {
    /// Run CARDIAC only.
    Off,
    /// Run both, and stop at the first card that differs.
    Check,
    /// Run both, and go on past a machine whose cards differ: each such machine is told once, where it first
    /// differs, and all of them again at the end (the frames are CARDIAC's).
    Report,
    /// Run the native build in place of CARDIAC (hand-written machines still run on CARDIAC). Fast, but checks only
    /// the frames against the reference.
    Only,
}

/// How a run is set up, chosen once by the demo before it loads any machine.
#[derive(Clone, Copy)]
pub struct Settings {
    /// Run each one-phase Rust machine's hand-written original beside it on the same cards, and compare.
    pub shadow: bool,
    pub native: Native,
    /// The demo's native builds, by machine name (the name of its source file, without .rs).
    pub find: fn(&str) -> Option<fn()>,
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// Choose the settings. Without this the host runs CARDIAC only, with no shadow runs.
pub fn set(s: Settings) { assert!(SETTINGS.set(s).is_ok(), "settings chosen twice"); }

/// The machines whose cards have differed so far (Report), each once.
static DIFFERED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Count a machine whose cards differed (Report): true the first time for each.
pub fn differed(name: &str) -> bool {
    let mut d = DIFFERED.lock().unwrap();
    if d.iter().any(|n| n == name) { false } else { d.push(name.to_string()); true }
}

/// The machines whose cards have differed so far (Report).
pub fn all_differed() -> Vec<String> { DIFFERED.lock().unwrap().clone() }

pub fn settings() -> Settings { *SETTINGS.get_or_init(|| Settings { shadow: false, native: Native::Off, find: |_| None }) }

/// The native build of the Rust machine NAME, when native runs are on.
pub fn find(name: &str) -> Option<fn()> {
    let s = settings();
    if s.native == Native::Off { None } else { (s.find)(name) }
}

struct Cards { deck: Vec<i32>, at: usize, punched: Vec<i32>, table: Vec<i32> }

thread_local! {
    static CARDS: RefCell<Cards> = const { RefCell::new(Cards { deck: Vec::new(), at: 0, punched: Vec::new(), table: Vec::new() }) };
}

/// Run machine `main` on `deck`, with `table` as what `load_resource()` returns (empty if the machine carries none),
/// and return the cards it punched. Every phase runs in turn, since `overlay()` does nothing natively, so `deck`
/// is every phase's cards one after another.
pub fn run(main: fn(), deck: &[i32], table: &[i32]) -> Vec<i32> {
    CARDS.with(|c| {
        let mut c = c.borrow_mut();
        c.deck.clear(); c.deck.extend_from_slice(deck); c.at = 0;
        c.punched.clear();
        c.table.clear(); c.table.extend_from_slice(table);
    });
    main();
    CARDS.with(|c| std::mem::take(&mut c.borrow_mut().punched))
}

/// The next card of the deck. Reading past its end is an error: on CARDIAC the machine would read whatever came next.
pub fn inp() -> i16 {
    CARDS.with(|c| {
        let mut c = c.borrow_mut();
        let Some(&v) = c.deck.get(c.at) else { panic!("read card {} of a deck of {}", c.at + 1, c.deck.len()) };
        c.at += 1;
        v as i16
    })
}

/// Punch a card. In a debug build a value outside -999..999, which CARDIAC can't punch, is an error.
pub fn out(v: i16) {
    debug_assert!((-999..=999).contains(&v), "punched {v}, which doesn't fit on a card (-999..999)");
    CARDS.with(|c| c.borrow_mut().punched.push(v as i32));
}

/// The table the host carries in the machine's program, as `load_resource::<N>(name)` returns it (a machine
/// carries at most one).
pub fn load_resource<const N: usize>(name: &str) -> [i16; N] {
    CARDS.with(|c| {
        let c = c.borrow();
        assert_eq!(c.table.len(), N, "the machine carries {name}, a table of {N} cards, the host gave {}", c.table.len());
        core::array::from_fn(|i| c.table[i] as i16)
    })
}
