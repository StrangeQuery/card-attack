//! The night arrival (the train's ending): the ride carried on into the night, the station, the drone's chase, the
//! airship and its volume knob going to 11, the stadium's lights, at 1536x768, every number worked out on CARDIAC.
//! The renderer is the train's (segments/train) with the night's parts (segments/night), written against the train's
//! operations; here each operation is done by the train's machines (fx::train::Cardiac), the host dealing the cards,
//! carrying what they punch and fetching from the printed tables by the indices they punch. `card-attack night
//! refs/night_ref` renders it and checks every frame against the reference (the same renderer done natively).
use crate::*;
use std::path::Path;
use super::train::{imp, Both, Cardiac};

/// The night's frames (0 .. F_HAND - 1; the stadium segment lays its render over the last DISSOLVE of them).
pub const FRAMES: usize = imp::F_HAND as usize;

/// The player's job before the night's blocks: the start (the night's world, the flight worked out, the shadows,
/// the lens's screens), kept for every block (live::keep).
pub fn setup(demo: &Path) {
    let mut o = Cardiac::new(demo);
    let world = imp::night_load(&mut o, demo.to_str().unwrap());
    live::count(o.host.steps);
    live::keep("night", world);
}

pub fn render(demo: &Path, refbin: &str) {
    let dir = demo.to_str().unwrap();
    if live::on() {
        let mut o = Cardiac::new(demo);
        let kept = live::kept::<imp::NightWorld>("night");
        let own;
        let world: &imp::NightWorld = match kept.as_deref() { Some(w) => w, None => { own = imp::night_load(&mut o, dir); &own } };
        const FRAME: usize = 1536 * 768 * 3;
        let mut bytes = Vec::with_capacity(live::range(FRAMES).len() * FRAME);
        for k in live::frames(FRAMES) { bytes.extend_from_slice(&imp::night_frame_v(&mut o, world, k as i64, true)); }
        live::count(o.host.steps);
        let path = demo.join("night/video_hd.rgb");
        match live::block() { Some((a, _)) => live::put_part(&path, a.min(FRAMES), FRAME, bytes), None => live::put(&path, bytes) }
        return;
    }
    if std::env::var("NIGHT_CAMERA").is_ok() {
        // the dissolve's camera worked out on CARDIAC (the start, then each frame's), checked against the
        // reference's, into data/camera_dissolve.txt
        let mut o = Cardiac::new(demo);
        let world = imp::night_load(&mut o, dir);
        let lines = imp::dissolve_camera(&mut o, &world);
        let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
        let reference = cardiac_host::reference(refbin, &[demo, Path::new("--camera")]);
        let same = text.as_bytes() == reference.as_slice();
        eprintln!("night camera: {} lines, {:.2}G instructions; identical to native Rust: {}", lines.len(), o.host.steps as f64 / 1e9, if same { "yes" } else { "NO" });
        assert!(same, "the camera differs from the reference's");
        std::fs::write(demo.join("night/data/camera_dissolve.txt"), text).unwrap();
        return;
    }
    // the party version (night/video_hd.rgb, the live cut's) unless NIGHT_FULL (night/video_full.rgb, for --hires);
    // the reference (this process's environment) draws the same
    let party = std::env::var("NIGHT_FULL").is_err();
    let only: Option<Vec<i64>> = std::env::var("NIGHT_FRAMES").ok().map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    if std::env::var("NIGHT_BOTH").is_ok() {
        let mut o = Both { n: imp::Native::new(imp::read_trig(dir)), c: Cardiac::new(demo), op: 0, last: 0, cost: Default::default() };
        let world = if std::env::var("NIGHT_BOTH_SETUP").is_ok() { imp::night_load(&mut o, dir) } else { imp::night_load(&mut o.n, dir) };
        o.last = o.c.host.steps;
        for &k in only.as_ref().expect("NIGHT_FRAMES") {
            imp::night_frame_v(&mut o, &world, k, party);
            eprintln!("frame {k}: the same, {} operations", o.op);
        }
        return;
    }
    const W: usize = 1536;
    const H: usize = 768;
    let mut o = Cardiac::new(demo);
    // NIGHT_SETUP_NATIVE: the start worked out natively (quicker, for measuring frames; not a checked render)
    let world = if std::env::var("NIGHT_SETUP_NATIVE").is_ok() { imp::night_load(&mut imp::Native::new(imp::read_trig(dir)), dir) } else { imp::night_load(&mut o, dir) };
    let s0 = o.host.steps;
    eprintln!("the start: {:.2}G", s0 as f64 / 1e9);
    let frames: Vec<i64> = (0..imp::F_HAND).filter(|k| only.as_ref().is_none_or(|l| l.contains(k))).collect();
    let mut video = cardiac_host::Video::<1536, 768>::new(frames.len());
    for (i, &k) in frames.iter().enumerate() {
        let s = o.host.steps;
        let img = imp::night_frame_v(&mut o, &world, k, party);
        video.buf[i * W * H * 3..(i + 1) * W * H * 3].copy_from_slice(&img);
        eprintln!("frame {k}: {:.2}G ({:.1}G so far)", (o.host.steps - s) as f64 / 1e9, o.host.steps as f64 / 1e9);
    }
    o.tally();
    o.host.done();
    if std::env::var("NIGHT_STAGES").is_ok() {
        for (s, c) in &o.by_stage { eprintln!("  {s:10} {:>9.1}M", *c as f64 / 1e6); }
        let mut v: Vec<_> = o.by_machine.iter().collect();
        v.sort_by_key(|(_, c)| std::cmp::Reverse(c.0));
        for ((s, m), (c, n)) in v.iter().take(60) { eprintln!("    {s:10} {m:12} {:>9.1}M {:>10} items {:>6.1} each", *c as f64 / 1e6, n, *c as f64 / (*n).max(1) as f64); }
    }
    let list = frames.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(",");
    let reference = cardiac_host::reference(refbin, &[demo, Path::new(&list)]);
    let out = std::env::var("NIGHT_OUT").map(std::path::PathBuf::from).unwrap_or_else(|_| demo.join(if party { "night/video_hd.rgb" } else { "night/video_full.rgb" }));
    finish("night", &o.host, &video, &reference, &out, if party { " (party: 768x384 world, 1536x768 subjects)" } else { " (full, 1536x768)" });
}
