//! The player: the film, shown live.
//!
//!     card-attack play [--windowed] [--lead SECONDS] [--workers N]   compute the cut live and show it, fullscreen
//!     card-attack play --dry-run [--workers N]                     compute it as scheduled, no window: each segment's slack
//!     card-attack play --dry-run --as-shown                         ... letting each frame go as the show would (for its memory)
//!     card-attack play --check [--workers N]                       ... and compare every frame with cut/video.rgb and the HD videos
//!     card-attack play --files [--windowed]                        show the rendered cut from disk (a stepping stone, not the show)
//! --space (macOS) goes fullscreen in a Space of its own instead of simple fullscreen (its frames were seen
//! to come up to 70 ms late).
//! --cut FILE plays another cut file under segments/ (its frames rendered beside it, for --check).
//! --from N starts the showing at frame N (as if the film had run up to it), --stop N ends it at frame N.
//! The soundtrack plays on the film's clock, decoded live by two CARDIACs (sound.rs); --audio-offset MS plays it MS ms
//! later (a display's lag),
//! --mute not at all.
//!
//! The film is a row of slots, one per frame of the cut, each filled once with its picture (192x96, or 1536x768 for
//! the HD segments); the presenter shows slot n at n/12 s from the clock's start.
pub mod live;
pub mod present;
pub mod sound;
pub mod render;

use crate::fx::cut::Cut;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// A frame's picture: w x h raw RGB, at `at` in a buffer it shares (a composed frame, or a frame of an HD video).
#[derive(Clone)]
pub struct Picture { pub w: usize, pub h: usize, buf: Arc<Vec<u8>>, at: usize }

impl Picture {
    pub fn new(w: usize, h: usize, buf: Arc<Vec<u8>>, at: usize) -> Self { assert!(at + w * h * 3 <= buf.len()); Picture { w, h, buf, at } }
    pub fn rgb(&self) -> &[u8] { &self.buf[self.at..self.at + self.w * self.h * 3] }
}

/// The film: a slot per frame, each filled once, from any thread, and emptied again once the frame has been shown
/// (`release`); when each was filed; and what each frame took, in CARDIAC instructions.
pub struct Film { slots: Vec<Mutex<Option<Picture>>>, ready: Vec<OnceLock<Instant>>, tally: Vec<OnceLock<u64>> }

impl Film {
    pub fn new(n: usize) -> Self { Film { slots: (0..n).map(|_| Mutex::new(None)).collect(), ready: (0..n).map(|_| OnceLock::new()).collect(), tally: (0..n).map(|_| OnceLock::new()).collect() } }
    /// The instructions frame n took (its share of each job it was made from, and its composing); set before the frame.
    pub fn set_tally(&self, n: usize, steps: u64) { let _ = self.tally[n].set(steps); }
    pub fn tally(&self, n: usize) -> Option<u64> { self.tally.get(n)?.get().copied() }
    pub fn len(&self) -> usize { self.slots.len() }
    pub fn is_empty(&self) -> bool { self.slots.is_empty() }
    /// Frame n's picture, if it has been filed and not yet released.
    pub fn get(&self, n: usize) -> Option<Picture> { self.slots.get(n)?.lock().unwrap().clone() }
    pub fn set(&self, n: usize, p: Picture) { assert!(self.ready[n].set(Instant::now()).is_ok(), "frame {n} filed twice"); *self.slots[n].lock().unwrap() = Some(p) }
    /// Whether frame n has been filed (it may have been released since).
    pub fn filed(&self, n: usize) -> bool { self.ready.get(n).is_some_and(|r| r.get().is_some()) }
    /// Let go of frame n's picture: it has been shown, and nothing needs it again. Its memory is freed (or, for a
    /// frame of a segment's video, the video's is, once its last frame lets go and the store has let it go too). The
    /// freeing happens on a thread of its own: handing back a whole video's memory takes long enough to make the
    /// presenter, which calls this, late for its next frame.
    pub fn release(&self, n: usize) {
        static GONE: OnceLock<std::sync::Mutex<std::sync::mpsc::Sender<Picture>>> = OnceLock::new();
        let gone = GONE.get_or_init(|| {
            let (tx, rx) = std::sync::mpsc::channel::<Picture>();
            std::thread::Builder::new().name("let go".into()).spawn(move || for p in rx { drop(p) }).unwrap();
            std::sync::Mutex::new(tx)
        });
        if let Some(p) = self.slots.get(n).and_then(|s| s.lock().unwrap().take()) { let _ = gone.lock().unwrap().send(p); }
    }
    /// When frame n was filed.
    pub fn ready_at(&self, n: usize) -> Option<Instant> { self.ready[n].get().copied() }
}

pub const HD: (usize, usize) = (1536, 768);

/// The rendered cut from disk: cut/video.rgb, and the HD frames hd.txt lists from their videos. Not the show (the
/// show computes every frame live); this only tries the window, the clock and the scaling.
fn from_files(demo: &Path) -> Film {
    let cut = Arc::new(std::fs::read(demo.join("cut/video.rgb")).expect("cut/video.rgb: render the cut first"));
    let frame = crate::FRAME2;
    let film = Film::new(cut.len() / frame);
    let mut hd: std::collections::HashMap<String, Arc<Vec<u8>>> = Default::default();
    let mut is_hd = vec![false; film.len()];
    for l in std::fs::read_to_string(demo.join("cut/hd.txt")).unwrap().lines() {
        let p: Vec<&str> = l.split(' ').collect();
        let (at, f): (usize, usize) = (p[0].parse().unwrap(), p[2].parse().unwrap());
        let v = hd.entry(p[1].into()).or_insert_with(|| Arc::new(std::fs::read(demo.join(p[1])).unwrap())).clone();
        film.set(at, Picture::new(HD.0, HD.1, v, f * HD.0 * HD.1 * 3));
        is_hd[at] = true;
    }
    for n in (0..film.len()).filter(|&n| !is_hd[n]) { film.set(n, Picture::new(crate::W2, crate::H2, cut.clone(), n * frame)); }
    film
}

fn arg<T: std::str::FromStr>(a: &[String], name: &str) -> Option<T> {
    a.iter().position(|s| s == name).map(|i| a.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or_else(|| panic!("{name} takes a value")))
}

/// The soundtrack, decoded up to `from` seconds in, ready to start on the clock; none if muted.
fn sound(a: &[String], from: f64) -> Option<sound::Prepared> {
    if a.iter().any(|s| s == "--mute") { return None }
    sound::prepare(from)
}

pub fn main(a: &[String]) {
    let demo = crate::demo_dir();
    let has = |f: &str| a.iter().any(|s| s == f);
    let windowed = has("--windowed");
    if has("--files") {
        eprintln!("play --files: the rendered cut from disk, NOT the show (the show computes every frame live)");
        let film = Arc::new(from_files(&demo));
        let lead: f64 = arg(a, "--lead").unwrap_or(1.0);
        let from: usize = arg(a, "--from").unwrap_or(0);
        let prepared = sound(a, from as f64 / present::FPS);
        let start = Instant::now() + Duration::from_secs_f64(lead) - Duration::from_secs_f64(from as f64 / present::FPS);
        let sound = prepared.and_then(|p| p.play(start, arg(a, "--audio-offset").unwrap_or(0.0)));
        let shown = present::show(film, from, arg(a, "--stop").unwrap_or(usize::MAX), start, windowed, has("--space"));
        report_shown(&shown);
        drop(sound);   // and say if any of it was missed
        return;
    }
    let workers: usize = arg(a, "--workers").unwrap_or_else(|| std::thread::available_parallelism().map_or(8, |n| n.get()).saturating_sub(2).max(1));
    let file: String = arg(a, "--cut").unwrap_or_else(|| "cut/cut.txt".into());
    let dir = crate::fx::cut::cut_dir(&file);
    let cut = Arc::new(Cut::parse(&demo, &crate::read(&demo, &file)));
    if has("--dry-run") || has("--check") {
        live::dry_run(&demo, cut, &dir, workers, has("--check"), has("--as-shown"));
        return;
    }
    let lead: f64 = arg(a, "--lead").unwrap_or(live::LEAD);
    let started = Instant::now();
    let run = live::Run::start(&demo, cut, &dir, workers, started);
    let sound = sound(a, 0.0).and_then(|p| p.play(started + Duration::from_secs_f64(lead), arg(a, "--audio-offset").unwrap_or(0.0)));
    let shown = present::show(run.film.clone(), 0, arg(a, "--stop").unwrap_or(usize::MAX), started + Duration::from_secs_f64(lead), windowed, has("--space"));
    report_shown(&shown);
    if let Some(&n) = shown.late.first() {
        let due = started + Duration::from_secs_f64(lead + n as f64 / present::FPS);
        match run.film.ready_at(n) {
            Some(r) => eprintln!("frame {n} was filed {:.2} s after its time ({:.2} s after start)", (r - due).as_secs_f64(), (r - started).as_secs_f64()),
            None => eprintln!("frame {n} still not filed"),
        }
    }
    drop(sound);   // and say if any of it was missed
    std::process::exit(if shown.late.is_empty() { 0 } else { 1 })   // don't wait for the workers
}

fn report_shown(s: &present::Shown) {
    let mut b = s.behind_ms.clone();
    b.sort_by(f64::total_cmp);
    if !b.is_empty() { eprintln!("frames up after their time: median {:.1} ms, 99% within {:.1} ms", b[b.len() / 2], b[b.len() * 99 / 100]); }
    let mut w = s.woke_ms.clone();
    w.sort_by(f64::total_cmp);
    if !w.is_empty() { eprintln!("woken after a frame's time: median {:.1} ms, 99% within {:.1} ms, at most {:.1}", w[w.len() / 2], w[w.len() * 99 / 100], w[w.len() - 1]); }
    match s.quit_at { Some(n) => eprintln!("quit at frame {n}"), None => eprintln!("shown: {} frames", s.frames) }
    if s.late.is_empty() { eprintln!("no frame late") } else { eprintln!("LATE: {} frames, the first {:?}", s.late.len(), &s.late[..s.late.len().min(10)]) }
    eprintln!("slowest draw {:.1} ms; a frame up at most {:.1} ms after its time (frame {}; a frame lasts {:.1} ms)", s.worst_draw_ms, s.worst_behind_ms, s.worst_behind_at, 1000.0 / present::FPS);
}
