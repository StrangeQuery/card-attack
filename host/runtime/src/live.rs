//! Live mode: the frames a demo's effects file go into memory instead of to disk, for a player that shows them as
//! they are computed. Off (the default), nothing changes: `finish` checks against the reference and saves, `fetch`
//! reads the file, `file` writes it. On (`live::start()`), `reference` runs nothing, `finish` puts the frames in the
//! store under the path it would have saved to, `fetch` takes a video another effect put there (the player runs
//! that effect first) and `file` puts the bytes there. The effects' own code paths stay the same either way.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

static LIVE: AtomicBool = AtomicBool::new(false);
static STORE: OnceLock<Mutex<HashMap<PathBuf, Arc<Vec<u8>>>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<PathBuf, Arc<Vec<u8>>>> { STORE.get_or_init(Default::default) }

/// Every machine's instructions and runs over the process's live run, added up as each Host is dropped (the player's
/// instruction share report).
static MACHINES: OnceLock<Mutex<HashMap<String, (u64, u64)>>> = OnceLock::new();
pub fn add_machines(per: &std::collections::BTreeMap<String, (u64, u64)>) {
    let mut m = MACHINES.get_or_init(Default::default).lock().unwrap();
    for (name, &(steps, runs)) in per { let e = m.entry(name.clone()).or_default(); e.0 += steps; e.1 += runs; }
}
/// The machines' totals so far: (name, instructions, runs), most instructions first.
pub fn machines() -> Vec<(String, u64, u64)> {
    let mut v: Vec<_> = MACHINES.get_or_init(Default::default).lock().unwrap().iter().map(|(k, &(s, r))| (k.clone(), s, r)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v
}

/// Turn live mode on, for the whole process.
pub fn start() { LIVE.store(true, Ordering::SeqCst) }
pub fn on() -> bool { LIVE.load(Ordering::Relaxed) }

/// Put frames (or any bytes) in the store under `path`.
pub fn put(path: &Path, bytes: Vec<u8>) { store().lock().unwrap().insert(path.to_path_buf(), Arc::new(bytes)); }

/// What the store holds under `path`, if anything.
pub fn get(path: &Path) -> Option<Arc<Vec<u8>>> { store().lock().unwrap().get(path).cloned() }

/// Take `path` out of the store, once nothing will read it again; its memory is freed when the last frame still
/// showing from it lets go.
pub fn remove(path: &Path) { store().lock().unwrap().remove(path); }

/// A video another effect files: from the store when live (it must be there already), else from disk.
pub fn fetch(path: &Path) -> Arc<Vec<u8>> {
    if on() {
        get(path).unwrap_or_else(|| panic!("live: {} isn't in memory yet (the player must run its effect first)", path.display()))
    } else {
        Arc::new(std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e} (render it first)", path.display())))
    }
}

/// Frames an effect files outside `finish`: into the store when live, else to disk.
pub fn file(path: &Path, bytes: Vec<u8>) {
    if on() { put(path, bytes) } else { std::fs::write(path, bytes).unwrap() }
}

thread_local! {
    /// Instructions run by the `finish`es on this thread since the last `take_steps`, for the player's accounts.
    static STEPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}
pub fn count(steps: u64) { STEPS.with(|s| s.set(s.get() + steps)) }
/// Instructions this thread's effects ran (as their `finish`es reported them), and start again from 0.
pub fn take_steps() -> u64 { STEPS.with(|s| s.replace(0)) }

// Blocks: the player may split an effect's frames into blocks, each run as a job of its own (on any worker), the
// effect's `render` making only that block's frames (`frames` says which). `finish` then files only those frames, as
// a part; the player joins the parts (`join`) when every block is done. Not live, or no block set, an effect makes
// all its frames as always.
thread_local! {
    static BLOCK: std::cell::Cell<Option<(usize, usize)>> = const { std::cell::Cell::new(None) };
    /// Seconds each frame took (as `frames` and `time_frames` saw), since the last `take_times`.
    static TIMES: std::cell::RefCell<Vec<(usize, f64)>> = const { std::cell::RefCell::new(Vec::new()) };
}
static PARTS: OnceLock<Mutex<HashMap<PathBuf, Vec<(usize, usize, Vec<u8>)>>>> = OnceLock::new();
fn parts() -> &'static Mutex<HashMap<PathBuf, Vec<(usize, usize, Vec<u8>)>>> { PARTS.get_or_init(Default::default) }

/// Make only frames from..to on this thread (None: all of them).
pub fn set_block(block: Option<(usize, usize)>) { BLOCK.with(|b| b.set(block)) }
/// The frames this thread is to make, if only some.
pub fn block() -> Option<(usize, usize)> { if on() { BLOCK.with(|b| b.get()) } else { None } }
/// The frames of an effect's n that this thread is to make: its block's (within 0..n), else all.
pub fn range(n: usize) -> std::ops::Range<usize> {
    match block() { Some((a, b)) => a.min(n)..b.min(n), None => 0..n }
}
/// Whether this thread is to make frame f.
pub fn wanted(f: usize) -> bool { block().is_none_or(|(a, b)| (a..b).contains(&f)) }

/// The frames of n this thread is to make (`range`), each one's time kept for the player's costs.
pub fn frames(n: usize) -> Frames { let r = range(n); Frames { next: r.start, end: r.end, at: None } }
pub struct Frames { next: usize, end: usize, at: Option<(usize, std::time::Instant)> }
impl Frames {
    fn close(&mut self) { if let Some((f, t)) = self.at.take() { if on() { TIMES.with(|v| v.borrow_mut().push((f, t.elapsed().as_secs_f64()))) } } }
}
impl Iterator for Frames {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        self.close();
        if self.next >= self.end { return None }
        let f = self.next;
        self.next += 1;
        self.at = Some((f, std::time::Instant::now()));
        Some(f)
    }
}
impl Drop for Frames { fn drop(&mut self) { self.close() } }

/// Run `work`, which makes frames `frames` together, its time shared out evenly between them.
pub fn time_frames<T>(frames: std::ops::Range<usize>, work: impl FnOnce() -> T) -> T {
    let t = std::time::Instant::now();
    let out = work();
    if on() && !frames.is_empty() {
        let each = t.elapsed().as_secs_f64() / frames.len() as f64;
        TIMES.with(|v| v.borrow_mut().extend(frames.map(|f| (f, each))));
    }
    out
}
/// The frames' times kept on this thread, and start again.
pub fn take_times() -> Vec<(usize, f64)> { TIMES.with(|v| std::mem::take(&mut *v.borrow_mut())) }

/// File frames from..from + n (`frame` bytes each) of the video at `path`, as a part.
pub fn put_part(path: &Path, from: usize, frame: usize, bytes: Vec<u8>) {
    assert_eq!(bytes.len() % frame, 0);
    parts().lock().unwrap().entry(path.to_path_buf()).or_default().push((from, frame, bytes));
}
/// Join the parts filed for `path`, in order (they must follow on from frame 0 without a gap), into the video the
/// store holds under it.
pub fn join(path: &Path) {
    let mut ps = parts().lock().unwrap().remove(path).unwrap_or_else(|| panic!("live: no parts of {}", path.display()));
    ps.sort_by_key(|p| p.0);
    let mut at = 0;
    let mut video = Vec::with_capacity(ps.iter().map(|p| p.2.len()).sum());
    for (from, frame, bytes) in ps {
        assert_eq!(from, at, "live: {} has a gap or an overlap at frame {at}", path.display());
        at += bytes.len() / frame;
        video.extend_from_slice(&bytes);
    }
    put(path, video);
}

// Kept: what an effect sets up once (on CARDIAC) for all its blocks, made by a job of its own before them, held by
// the host as bookkeeping and handed to each block.
type Kept = HashMap<String, Arc<dyn std::any::Any + Send + Sync>>;
static KEPT: OnceLock<Mutex<Kept>> = OnceLock::new();
fn kept_store() -> &'static Mutex<Kept> { KEPT.get_or_init(Default::default) }
/// Keep `value` under `key`, for the blocks.
pub fn keep<T: std::any::Any + Send + Sync>(key: &str, value: T) { kept_store().lock().unwrap().insert(key.to_string(), Arc::new(value)); }
/// What was kept under `key`, live, if anything.
pub fn kept<T: std::any::Any + Send + Sync>(key: &str) -> Option<Arc<T>> {
    if !on() { return None }
    kept_store().lock().unwrap().get(key).cloned().map(|v| v.downcast::<T>().unwrap_or_else(|_| panic!("live: {key} kept as another type")))
}
