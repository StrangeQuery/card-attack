//! What a demo on CARDIAC needs from its host, whatever the demo: loading machines and their phases, running them
//! with their cards, counting instructions, shadow runs (a hand-written original beside its Rust port) and native
//! runs (a Rust machine built natively beside its CARDIAC tape), checking frames against a reference renderer, and
//! writing the video. The demo itself decides what each machine is dealt and what becomes of what it punches.
//!
//! A demo is laid out as folders, one per part, each with its Rust machines in `machines/`
//! (`PART/machines/NAME.rs`, compiled next to it: its tapes, NAME.cqb, NAME.2.cqb ..., and NAME.cqb.meta) and its
//! hand-written ones wherever it likes; and `shared/none_data.asm`, the empty data block a one-phase hand-written
//! machine is assembled on.
pub use cargo_cardiac::{asm, cardiac, listing, tape};   // the emulator, the assembler, tapes: rust-cardiac's
pub mod overlay;
pub mod native;
pub mod live;

use cardiac::{run, Stop};
pub use overlay::{build, carrying, compiled, hand, machine_file, one, patched, read, rust, rust_or, Compiled, Machine};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

/// Runs machines and keeps count: instructions and runs per machine.
pub struct Host { pub steps: u64, pub runs: u64, pub per: BTreeMap<String, (u64, u64)>, out: Vec<i32>, flat: Vec<i32>, started: Instant, took: Option<f64>,
    /// When set, every distinct program loaded (as the boot loader shows it) is kept, in the order first run.
    pub record: Option<(Vec<Vec<i32>>, std::collections::HashSet<Vec<i32>>)>,
    /// Runs of machines that have a native build, and the seconds spent on them on CARDIAC and natively.
    pub native: NativeRuns }

#[derive(Default, Clone, Copy)]
pub struct NativeRuns { pub runs: u64, pub cardiac_secs: f64, pub native_secs: f64 }

impl Default for Host { fn default() -> Self { Self::new() } }

// live, each Host's per-machine counts go into the process's totals as it goes (see live::machines)
impl Drop for Host { fn drop(&mut self) { if live::on() && !self.per.is_empty() { live::add_machines(&self.per) } } }

impl Host {
    pub fn new() -> Self { Host { steps: 0, runs: 0, per: BTreeMap::new(), out: Vec::with_capacity(1024), flat: Vec::new(), started: Instant::now(), took: None, record: None, native: NativeRuns::default() } }

    /// Run machine `m`: each phase's tape followed by that phase's data cards (`decks[i]`, any number of pieces).
    /// Panics unless it halts, and unless it punches exactly `n` cards when `n` is given.
    pub fn go(&mut self, m: &Machine, decks: &[&[&[i32]]], n: Option<usize>) -> Vec<i32> {
        self.run_machine(m, decks, n);
        self.out.clone()
    }
    /// `go`, the punched cards put on the end of `out` (no new vector for them).
    pub fn go_into(&mut self, m: &Machine, decks: &[&[&[i32]]], n: Option<usize>, out: &mut Vec<i32>) {
        self.run_machine(m, decks, n);
        out.extend_from_slice(&self.out);
    }
    /// `go1` and `gon`, onto the end of `out`.
    pub fn go1_into(&mut self, m: &Machine, deck: &[&[i32]], n: usize, out: &mut Vec<i32>) { self.go_into(m, &[deck], Some(n), out) }
    pub fn gon_into(&mut self, m: &Machine, deck: &[&[i32]], out: &mut Vec<i32>) { self.go_into(m, &[deck], None, out) }

    /// Run a machine as `go` says, its cards left in `self.out`.
    fn run_machine(&mut self, m: &Machine, decks: &[&[&[i32]]], n: Option<usize>) {
        let split = !m.split.is_empty() && decks.len() == 1;
        // the reader's parts, on the stack when they are few (they nearly always are)
        let count = if split { 2 * m.tapes.len() } else { m.tapes.len() + decks.iter().take(m.tapes.len()).map(|d| d.len()).sum::<usize>() };
        let mut stack: [&[i32]; 24] = [&[]; 24];
        let mut heap: Vec<&[i32]>;
        let parts: &mut [&[i32]] = if count <= stack.len() { &mut stack[..count] } else { heap = vec![&[]; count]; &mut heap };
        // a machine in phases dealt one deck: it is dealt out from one copy, kept between runs
        let mut flat = std::mem::take(&mut self.flat);
        flat.clear();
        if split { for p in decks[0] { flat.extend_from_slice(p); } }
        let mut k = 0;
        if split {
            // a compiled machine in phases: deal the one deck out, so many cards after each phase's tape
            let mut at = 0;
            for (i, t) in m.tapes.iter().enumerate() {
                parts[k] = t; k += 1;
                let n = if i < m.split.len() { m.split[i] } else { flat.len() - at };
                parts[k] = &flat[at..at + n]; k += 1; at += n;
            }
        } else {
            for (i, t) in m.tapes.iter().enumerate() {
                parts[k] = t; k += 1;
                if let Some(d) = decks.get(i) { for p in d.iter() { parts[k] = p; k += 1; } }
            }
        }
        debug_assert_eq!(k, count);
        let parts: &[&[i32]] = parts;
        if let Some((progs, seen)) = &mut self.record {
            // the loader cards, each address and the word punched into it (skipping cells only cleared), the start
            let t = &m.tapes[0];
            let mut p = vec![t[0], t[1]];
            let mut i = 2;
            // (the start: Brian Stuart's pair 002 8xx, or a single 8xx card)
            while i + 1 < t.len() && t[i] != 2 && !(800..900).contains(&t[i]) { if t[i + 1] != 0 { p.push(t[i]); p.push(t[i + 1]); } i += 2; }
            p.extend_from_slice(&t[i..t.len().min(i + if t[i] == 2 { 2 } else { 1 })]);
            if seen.insert(p.clone()) { progs.push(p); }
        }
        // natively, every phase's cards one after another (overlay() does nothing there, so one call runs them all)
        let native = m.native.map(|f| (f, if split { flat.clone() } else { decks.iter().take(m.tapes.len()).flat_map(|d| d.concat()).collect::<Vec<i32>>() }));
        let mut steps = 0;
        if let (Some((f, deck)), native::Native::Only) = (&native, native::settings().native) {
            let t = Instant::now();
            self.out = native::run(*f, deck, &m.table);
            self.native.native_secs += t.elapsed().as_secs_f64(); self.native.runs += 1;
            if n.is_some_and(|n| self.out.len() != n) { panic!("{} natively: punched {} cards", m.name, self.out.len()); }
        } else {
            self.out.clear();
            let t = native.is_some().then(Instant::now);
            // a machine in phases is in overlay mode: its stop for tape n (HRS n) runs on into the tape the deck holds
            let (why, s) = if m.tapes.len() > 1 { cardiac::run_stacked(&parts, &mut self.out, u64::MAX) } else { run(&parts, &mut self.out, u64::MAX) };
            steps = s;
            if why != Stop::Halt || n.is_some_and(|n| self.out.len() != n) {
                panic!("{}: {:?} after punching {} cards", m.name, why, self.out.len());
            }
            if let (Some((f, deck)), Some(t)) = (&native, t) {
                self.native.cardiac_secs += t.elapsed().as_secs_f64();
                let t = Instant::now();
                let got = native::run(*f, deck, &m.table);
                self.native.native_secs += t.elapsed().as_secs_f64(); self.native.runs += 1;
                if got != self.out {
                    let at = got.iter().zip(&self.out).position(|(a, b)| a != b).unwrap_or(got.len().min(self.out.len()));
                    if native::settings().native == native::Native::Report {
                        if native::differed(&m.name) {
                            let near = |v: &[i32]| v[at.saturating_sub(3)..(at + 6).min(v.len())].to_vec();
                            eprintln!("{} punches other cards natively than on CARDIAC, from card {}: deck {:?}\n CARDIAC {:?}\n native  {:?} (cards {} on)",
                                m.name, at + 1, deck, near(&self.out), near(&got), at.saturating_sub(3) + 1);
                        }
                    } else {
                        panic!("{} punches other cards natively than on CARDIAC, from card {}: deck {:?}\n CARDIAC {:?}\n native  {:?}", m.name, at + 1, deck, self.out, got);
                    }
                }
            }
            if let Some(sh) = &m.shadow {
                // the hand-written original on the same cards, all of them after its one tape
                let mut p2: Vec<&[i32]> = vec![&sh.tapes[0]];
                for d in decks { p2.extend_from_slice(d); }
                let mut o2 = vec![];
                let (w2, _) = run(&p2, &mut o2, u64::MAX);
                if o2 != self.out || w2 != why {
                    let deck: Vec<i32> = decks.iter().flat_map(|d| d.iter().flat_map(|p| p.iter().copied())).collect();
                    panic!("{} differs from {}: deck {:?}\n rust {:?}\n hand {:?}", m.name, sh.name, deck, self.out, o2);
                }
            }
        }
        self.steps += steps; self.runs += 1;
        match self.per.get_mut(&m.name) {
            Some(e) => { e.0 += steps; e.1 += 1; }
            None => { self.per.insert(m.name.clone(), (steps, 1)); }
        }
        self.flat = flat;
    }
    /// The common case: one phase, its data cards in pieces, punching exactly n cards.
    pub fn go1(&mut self, m: &Machine, deck: &[&[i32]], n: usize) -> Vec<i32> { self.go(m, &[deck], Some(n)) }
    /// One phase, any number of cards punched.
    pub fn gon(&mut self, m: &Machine, deck: &[&[i32]]) -> Vec<i32> { self.go(m, &[deck], None) }

    /// Seconds spent computing: up to `done()` if called (before the reference runs), else until now.
    pub fn secs(&self) -> f64 { self.took.unwrap_or_else(|| self.started.elapsed().as_secs_f64()) }
    pub fn done(&mut self) { self.took = Some(self.started.elapsed().as_secs_f64()); }

    /// PROFILE=file appends this effect's per-machine counts, one JSON line, for the instruction share report.
    pub fn profile(&self, effect: &str) {
        if let Ok(f) = std::env::var("PROFILE") {
            use std::io::Write;
            let body: Vec<String> = self.per.iter().map(|(k, (s, r))| format!("\"{k}\":{{\"steps\":{s},\"runs\":{r}}}")).collect();
            let mut fh = std::fs::OpenOptions::new().create(true).append(true).open(f).unwrap();
            writeln!(fh, "{{\"script\":\"{effect}\",\"machines\":{{{}}}}}", body.join(",")).unwrap();
        }
    }
}

/// Frames of W x H pixels, raw RGB, back to back.
pub struct Video<const W: usize, const H: usize> { pub frames: usize, pub buf: Vec<u8> }

impl<const W: usize, const H: usize> Video<W, H> {
    const FRAME: usize = W * H * 3;
    pub fn new(frames: usize) -> Self { Video { frames, buf: vec![0; frames * Self::FRAME] } }
    /// File one row of punched colours (r, g, b per pixel, each 0-255) into frame f.
    pub fn row(&mut self, f: usize, y: usize, rgb: &[i32]) {
        assert_eq!(rgb.len(), W * 3);
        let at = (f * H + y) * W * 3;
        for (i, &v) in rgb.iter().enumerate() { self.buf[at + i] = v as u8; }
    }
    pub fn frame(&self, f: usize) -> &[u8] { &self.buf[f * Self::FRAME..(f + 1) * Self::FRAME] }

    /// Compare against the reference renderer's output: (frames identical, first bad frame).
    pub fn check(&self, reference: &[u8]) -> (usize, Option<usize>) {
        let mut good = 0; let mut first = None;
        for f in 0..self.frames {
            if reference.get(f * Self::FRAME..(f + 1) * Self::FRAME) == Some(self.frame(f)) { good += 1 } else if first.is_none() { first = Some(f) }
        }
        (good, first)
    }
    pub fn save(&self, path: &Path) { std::fs::write(path, &self.buf).unwrap(); }
}

/// Run a reference renderer (a native Rust program written independently of the machines) and take its frames.
/// Live (see `live`), no reference runs: the frames go to the player unchecked, and `finish` takes an empty one.
pub fn reference(bin: &str, args: &[&Path]) -> Vec<u8> {
    if live::on() { return vec![] }
    let out = std::process::Command::new(bin).args(args).output().unwrap_or_else(|e| panic!("{bin}: {e}"));
    assert!(out.status.success(), "{bin} failed");
    out.stdout
}

/// The usual ending: check against the reference, save, and report. Live (see `live`): the frames go into the store
/// under `save_to`, unchecked and unsaved (a block's frames only, as a part, when the player split the effect).
pub fn finish<const W: usize, const H: usize>(effect: &str, host: &Host, video: &Video<W, H>, reference: &[u8], save_to: &Path, extra: &str) {
    if live::on() {
        live::count(host.steps);
        // a block of the effect's frames (see live): only those, as a part
        match live::block() {
            Some((a, b)) => { let (a, b) = (a.min(video.frames), b.min(video.frames)); live::put_part(save_to, a, Video::<W, H>::FRAME, video.buf[a * Video::<W, H>::FRAME..b * Video::<W, H>::FRAME].to_vec()) }
            None => live::put(save_to, video.buf.clone()),
        }
        return
    }
    let (good, first) = video.check(reference);
    video.save(save_to);
    host.profile(effect);
    let nr = host.native;
    let native = match native::settings().native {
        native::Native::Off => String::new(),
        native::Native::Check => format!("; {} runs also native, same cards (CARDIAC {:.2}s, native {:.3}s)", nr.runs, nr.cardiac_secs, nr.native_secs),
        native::Native::Report => match native::all_differed() {
            d if d.is_empty() => format!("; {} runs also native, same cards (CARDIAC {:.2}s, native {:.3}s)", nr.runs, nr.cardiac_secs, nr.native_secs),
            d => format!("; {} runs also native, and these machines' cards differ: {}", nr.runs, d.join(", ")),
        },
        native::Native::Only => format!("; {} runs native only ({:.3}s)", nr.runs, nr.native_secs),
    };
    println!("{effect}: {} frames, {} machine runs, {}M instructions in {:.2}s{extra}{native}; identical to native Rust: {good}/{}{}",
        video.frames, host.runs, host.steps / 1_000_000, host.secs(), video.frames, first.map(|f| format!(" first bad {f}")).unwrap_or_default());
    if good != video.frames { std::process::exit(1); }
}

/// Numbers from a text file, whitespace separated.
pub fn numbers(path: &Path) -> Vec<i32> {
    std::fs::read_to_string(path).unwrap().split_whitespace().map(|v| v.parse().unwrap()).collect()
}
