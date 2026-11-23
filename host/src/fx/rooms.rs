//! The rooms: the dorm (out of Mode X's CRT into a dorm room, Dope on its monitor, back in) and
//! the screening room (out of the projected drive into a hall, THE END, the prompt, SEA++ loading, the dive starting
//! on the screen, back in), at 1536x768. The renderer is the segment's (segments/rooms/rooms.rs), written against the
//! train's operations; here each operation is done by the train's machines (fx::train::Cardiac), the host fetching
//! each pixel from the printed tables and the pictures by the indices they punch. `card-attack rooms refs/rooms_ref`
//! renders it and checks every frame against the reference (the same renderer done natively).
use crate::*;
use std::path::Path;

#[allow(dead_code, clippy::all, unused_variables, unused_mut, unused_imports)]
pub mod imp {
    pub use super::super::train::imp::{fl, md, n_of, v, Native, Ops, Tab, A, CARD, K, LONG, V, WIDE};
    include!("../../../segments/rooms/rooms.rs");
}

pub const FRAMES: usize = imp::FRAMES as usize;
const FRAME: usize = 1536 * 768 * 3;
pub const VIDEO: &str = "rooms/video_hd.rgb";
/// The videos the rooms show on their screens.
pub const READS: [&str; 5] = ["modex/video_hd.rgb", "polys/video.rgb", "accel/video_hd.rgb", "deep/video_hd.rgb", "ui/accelbrag.rgba"];

/// The frames of the videos the screens show (from the store when live, else from disk).
fn srcs(demo: &Path) -> imp::Srcs {
    let part = |v: &str, size: usize, a: i64, n: i64| -> Vec<u8> {
        let b = live::fetch(&demo.join(v));
        b[a as usize * size..(a + n) as usize * size].to_vec()
    };
    imp::Srcs {
        modex: part(READS[0], FRAME, imp::MODEX0, imp::MODEX_N),
        polys: part(READS[1], 192 * 96 * 3, imp::POLYS0, imp::POLYS_N),
        drive: part(READS[2], FRAME, imp::DRIVE0, imp::DRIVE_N),
        dive: part(READS[3], FRAME, 0, imp::DIVE_N),
        brag: live::fetch(&demo.join(READS[4])).to_vec(),
    }
}

/// The player's job before the rooms' blocks: the printed tables, kept for every block.
pub fn setup(demo: &Path) {
    live::keep("rooms", imp::load(demo.to_str().unwrap()));
}

pub fn render(demo: &Path, refbin: &str) {
    let dir = demo.to_str().unwrap();
    if live::on() {
        let kept = live::kept::<imp::World>("rooms");
        let own;
        let w: &imp::World = match kept.as_deref() { Some(w) => w, None => { own = imp::load(dir); &own } };
        let s = srcs(demo);
        let mut o = super::train::Cardiac::new(demo);
        let range = live::range(FRAMES);
        let mut bytes = Vec::with_capacity(range.len() * FRAME);
        for k in live::frames(FRAMES) { bytes.extend_from_slice(&imp::render(&mut o, w, &s, k as i64)); }
        live::count(o.host.steps);
        let path = demo.join(VIDEO);
        match live::block() { Some((a, _)) => live::put_part(&path, a.min(FRAMES), FRAME, bytes), None => live::put(&path, bytes) }
        return;
    }
    let w = imp::load(dir);
    let s = srcs(demo);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).saturating_sub(2).max(1);
    let started = std::time::Instant::now();
    let reference = cardiac_host::reference(refbin, &[demo]);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: Vec<std::sync::Mutex<Option<(Vec<u8>, u64, u64)>>> = (0..FRAMES).map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| {
                let mut o = super::train::Cardiac::new(demo);
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if k >= FRAMES { break; }
                    let (s0, r0) = (o.host.steps, o.host.runs);
                    let f = imp::render(&mut o, &w, &s, k as i64);
                    *results[k].lock().unwrap() = Some((f, o.host.steps - s0, o.host.runs - r0));
                }
            });
        }
    });
    let (mut steps, mut runs, mut good, mut first_bad) = (0u64, 0u64, 0, None);
    let mut video = Vec::with_capacity(FRAMES * FRAME);
    for (k, r) in results.into_iter().enumerate() {
        let (f, st, ru) = r.into_inner().unwrap().unwrap();
        steps += st;
        runs += ru;
        let same = reference.len() >= (k + 1) * FRAME && reference[k * FRAME..(k + 1) * FRAME] == f[..];
        if same { good += 1; } else if first_bad.is_none() { first_bad = Some(k); }
        video.extend_from_slice(&f);
    }
    std::fs::write(demo.join(VIDEO), &video).unwrap();
    println!("rooms: {FRAMES} frames, {runs} machine runs, {}M instructions in {:.2}s (1536x768); identical to native Rust: {good}/{FRAMES}{}",
        steps / 1_000_000, started.elapsed().as_secs_f64(), first_bad.map(|f| format!(" first bad {f}")).unwrap_or_default());
    if good != FRAMES { std::process::exit(1); }
}
