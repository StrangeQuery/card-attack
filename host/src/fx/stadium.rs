//! The stadium party (the film's last segment): 65,536 people dancing in a stadium at night at 1536x768, every number
//! worked out on CARDIAC. The renderer is the segment's (segments/stadium/*.rs), written against the train's
//! operations; here each operation is done by the train's machines (fx::train::Cardiac), the host dealing the cards,
//! carrying what they punch, fetching from the printed tables by the indices they punch, picking by the flags they
//! punch, sorting pieces to their pixels. `card-attack stadium refs/stadium_ref` renders it and checks every frame
//! against the reference (the same renderer with the operations done natively).
//!
//! The world is drawn at 768 x 384 and the subjects at 1536 x 768 (segments/stadium/party.rs), into
//! `stadium/video_hd.rgb`, dissolving from `night/video_hd.rgb`.
use crate::*;
use std::path::Path;
use super::train::imp::Ops as _;

#[allow(dead_code, clippy::all, unused_variables, unused_mut, unused_imports)]
pub mod imp {
    pub use super::super::train::imp::{fl, md, n_of, v, Native, Ops, Tab, A, CARD, K, LONG, V, WIDE};
    include!("../../../segments/stadium/scene.rs");
    include!("../../../segments/stadium/maths.rs");
    include!("../../../segments/stadium/clock.rs");
    include!("../../../segments/stadium/raster.rs");
    include!("../../../segments/stadium/light.rs");
    include!("../../../segments/stadium/pictures.rs");
    include!("../../../segments/stadium/crowd.rs");
    include!("../../../segments/stadium/shade.rs");
    include!("../../../segments/stadium/glow.rs");
    include!("../../../segments/stadium/beams.rs");
    include!("../../../segments/stadium/air.rs");
    include!("../../../segments/stadium/things.rs");
    include!("../../../segments/stadium/frame.rs");
    include!("../../../segments/stadium/party.rs");
    include!("../../../segments/stadium/inputs.rs");
}

pub const FRAMES: usize = imp::FRAMES as usize;
const FRAME: usize = 1536 * 768 * 3;

/// The party version (the default) or the full one (STADIUM_FULL).
pub fn hd() -> bool { std::env::var("STADIUM_FULL").map_or(true, |v| v.is_empty() || v == "0") }
/// The video this version writes, and the night's it dissolves from.
pub fn video() -> &'static str { if hd() { "stadium/video_hd.rgb" } else { "stadium/video_full.rgb" } }
fn night_video() -> &'static str { if hd() { "night/video_hd.rgb" } else { "night/video_full.rgb" } }

/// What the frames need besides the world: the night arrival's frames (the dissolve's), its camera.
struct Ins { night: Option<Vec<u8>>, cams: imp::NightCams }

fn inputs(demo: &Path, need_night: bool) -> Ins {
    let cams = imp::night_cams(demo.to_str().unwrap());
    let night = if need_night { Some(imp::night_dissolve_frames(&live::fetch(&demo.join(night_video())), &cams)) } else { None };
    Ins { night, cams }
}

/// The player's job before the stadium's blocks: the world (the scene, the printed tables, the screen's numbers
/// worked out on CARDIAC), kept for every block.
pub fn setup(demo: &Path) {
    let mut o = super::train::Cardiac::new(demo);
    let w = imp::load(&mut o, demo.to_str().unwrap());
    live::count(o.host.steps);
    live::keep("stadium", w);
}

fn frame(o: &mut super::train::Cardiac, w: &imp::World, ins: &Ins, k: usize) -> Vec<u8> {
    let inp = imp::Inputs { night: ins.night.as_deref(), night_cam: &ins.cams, hd: hd() };
    imp::render(o, w, &inp, k as i64)
}

pub fn render(demo: &Path, refbin: &str) {
    if live::on() {
        let mut o = super::train::Cardiac::new(demo);
        let kept = live::kept::<imp::World>("stadium");
        let own;
        let w: &imp::World = match kept.as_deref() { Some(w) => w, None => { own = imp::load(&mut o, demo.to_str().unwrap()); &own } };
        let range = live::range(FRAMES);
        let ins = inputs(demo, range.start < imp::DISSOLVE as usize);
        let mut bytes = Vec::with_capacity(range.len() * FRAME);
        for k in live::frames(FRAMES) { bytes.extend_from_slice(&frame(&mut o, w, &ins, k)); }
        live::count(o.host.steps);
        let path = demo.join(video());
        match live::block() { Some((a, _)) => live::put_part(&path, a.min(FRAMES), FRAME, bytes), None => live::put(&path, bytes) }
        return;
    }
    let dir = demo.to_str().unwrap();
    if std::env::var("STADIUM_BOTH").is_ok() {
        // every operation both ways, stopping at the first that differs (STADIUM_FRAMES)
        let mut o = super::train::Both { n: super::train::imp::Native::new(imp::read_trig(dir)), c: super::train::Cardiac::new(demo), op: 0, last: 0, cost: Default::default() };
        let w = imp::load(&mut o.n, dir);
        o.last = o.c.host.steps;
        for k in std::env::var("STADIUM_FRAMES").unwrap().split(',').map(|v| v.parse::<usize>().unwrap()) {
            let ins = inputs(demo, k < imp::DISSOLVE as usize);
            let inp = imp::Inputs { night: ins.night.as_deref(), night_cam: &ins.cams, hd: hd() };
            imp::render(&mut o, &w, &inp, k as i64);
            eprintln!("frame {k}: the same, {} operations", o.op);
        }
        if let Ok(n) = std::env::var("STADIUM_OPS") {
            let mut v: Vec<_> = o.cost.iter().collect();
            v.sort_by_key(|(_, c)| std::cmp::Reverse(c.0));
            for ((s, w), (c, n_)) in v.iter().take(n.parse().unwrap_or(40)) { eprintln!("  {s:10} {w:34} {:>8.1}M {n_:>6}", *c as f64 / 1e6); }
        }
        return;
    }
    let only: Option<Vec<usize>> = std::env::var("STADIUM_FRAMES").ok().map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let frames: Vec<usize> = (0..FRAMES).filter(|k| only.as_ref().is_none_or(|l| l.contains(k))).collect();
    let threads: usize = std::env::var("STADIUM_THREADS").ok().map_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()).saturating_sub(2).max(1), |s| s.parse().unwrap());
    let mut o = super::train::Cardiac::new(demo);
    let w = imp::load(&mut o, dir);
    let ins = inputs(demo, frames.iter().any(|&k| k < imp::DISSOLVE as usize));
    let started = std::time::Instant::now();
    let mut steps = o.host.steps;
    let mut runs = o.host.runs;
    let mut by_stage: std::collections::BTreeMap<&'static str, u64> = Default::default();
    let mut by_machine: std::collections::BTreeMap<(&'static str, &'static str), (u64, u64)> = Default::default();
    let mut good = 0;
    let mut first_bad = None;
    let mut out = std::fs::File::create(demo.join(video())).unwrap();
    // in chunks: the reference's frames, CARDIAC's on the threads, compared, filed
    for chunk in frames.chunks(threads.max(1) * 2) {
        let list = chunk.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(",");
        let reference = cardiac_host::reference(refbin, &[demo, Path::new(&list)]);
        let next = std::sync::atomic::AtomicUsize::new(0);
        let results: Vec<std::sync::Mutex<Option<(Vec<u8>, u64, u64, std::collections::BTreeMap<&'static str, u64>, std::collections::BTreeMap<(&'static str, &'static str), (u64, u64)>)>>> = chunk.iter().map(|_| std::sync::Mutex::new(None)).collect();
        std::thread::scope(|s| {
            for _ in 0..threads.min(chunk.len()) {
                s.spawn(|| {
                    let mut o = super::train::Cardiac::new(demo);
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        if i >= chunk.len() { break; }
                        let (s0, r0) = (o.host.steps, o.host.runs);
                        let f = frame(&mut o, &w, &ins, chunk[i]);
                        o.stage("end");
                        let st = std::mem::take(&mut o.by_stage);
                        let bm = std::mem::take(&mut o.by_machine);
                        *results[i].lock().unwrap() = Some((f, o.host.steps - s0, o.host.runs - r0, st, bm));
                    }
                });
            }
        });
        for (i, r) in results.into_iter().enumerate() {
            let (f, st, ru, bs, bm) = r.into_inner().unwrap().unwrap();
            steps += st;
            runs += ru;
            for (k, c) in bs { *by_stage.entry(k).or_insert(0) += c; }
            for (k, c) in bm { let e = by_machine.entry(k).or_insert((0, 0)); e.0 += c.0; e.1 += c.1; }
            let same = reference.len() >= (i + 1) * FRAME && reference[i * FRAME..(i + 1) * FRAME] == f[..];
            if same { good += 1; } else if first_bad.is_none() { first_bad = Some(chunk[i]); }
            std::io::Write::write_all(&mut out, &f).unwrap();
            eprintln!("frame {}: {:.2}G{}", chunk[i], st as f64 / 1e9, if same { "" } else { "  DIFFERS" });
        }
    }
    if std::env::var("STADIUM_STAGES").is_ok() {
        for (s, c) in &by_stage { eprintln!("  {s:10} {:>9.1}M a frame", *c as f64 / 1e6 / frames.len() as f64); }
        let mut v: Vec<_> = by_machine.iter().collect();
        v.sort_by_key(|(_, c)| std::cmp::Reverse(c.0));
        for ((s, m), (c, n)) in v.iter().take(40) { eprintln!("    {s:10} {m:12} {:>9.1}M {:>10} items {:>6.1} each", *c as f64 / 1e6 / frames.len() as f64, n, *c as f64 / (*n).max(1) as f64); }
    }
    println!("stadium ({}): {} frames, {} machine runs, {}M instructions in {:.2}s (1536x768); identical to native Rust: {good}/{}{}",
        if hd() { "party version" } else { "full version" }, frames.len(), runs, steps / 1_000_000, started.elapsed().as_secs_f64(), frames.len(), first_bad.map(|f| format!(" first bad {f}")).unwrap_or_default());
    if good != frames.len() { std::process::exit(1); }
}
