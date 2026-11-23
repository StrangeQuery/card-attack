//! The ending's SQ: after the CARDIAC on the giant screen, a hard cut to a big SQ in silver
//! chrome seen from the ground, STRANGE QUERY materialising under it, 36 frames at 1536x768. The renderer is the
//! segment's (segments/sqend/sqend.rs), written against the train's operations; here each is done by the train's
//! machines (fx::train::Cardiac): the picture's chrome worked out once (the start), the name brought in each frame.
//! `card-attack sqend refs/sqend_ref` renders it and checks every frame against the reference.
use crate::*;
use std::path::Path;

#[allow(dead_code, clippy::all, unused_variables, unused_mut, unused_imports)]
pub mod imp {
    pub use super::super::train::imp::{fl, md, n_of, v, Native, Ops, Tab, A, CARD, K, LONG, V, WIDE};
    include!("../../../segments/sqend/sqend.rs");
}

pub const FRAMES: usize = imp::FRAMES as usize;
const FRAME: usize = 1536 * 768 * 3;
pub const VIDEO: &str = "sqend/video_hd.rgb";

/// The player's job before the blocks: the start (the picture's chrome on CARDIAC), kept for every block.
pub fn setup(demo: &Path) {
    let mut o = super::train::Cardiac::new(demo);
    let w = imp::load(&mut o, demo.to_str().unwrap());
    live::count(o.host.steps);
    live::keep("sqend", w);
}

pub fn render(demo: &Path, refbin: &str) {
    let dir = demo.to_str().unwrap();
    let mut o = super::train::Cardiac::new(demo);
    if live::on() {
        let kept = live::kept::<imp::World>("sqend");
        let own;
        let w: &imp::World = match kept.as_deref() { Some(w) => w, None => { own = imp::load(&mut o, dir); &own } };
        let range = live::range(FRAMES);
        let mut bytes = Vec::with_capacity(range.len() * FRAME);
        for k in live::frames(FRAMES) { bytes.extend_from_slice(&imp::render(&mut o, w, k as i64)); }
        live::count(o.host.steps);
        let path = demo.join(VIDEO);
        match live::block() { Some((a, _)) => live::put_part(&path, a.min(FRAMES), FRAME, bytes), None => live::put(&path, bytes) }
        return;
    }
    let started = std::time::Instant::now();
    let w = imp::load(&mut o, dir);
    let mut video = Vec::with_capacity(FRAMES * FRAME);
    for k in 0..FRAMES { video.extend_from_slice(&imp::render(&mut o, &w, k as i64)); }
    let reference = cardiac_host::reference(refbin, &[demo]);
    let good = (0..FRAMES).filter(|&k| reference.len() >= (k + 1) * FRAME && reference[k * FRAME..(k + 1) * FRAME] == video[k * FRAME..(k + 1) * FRAME]).count();
    std::fs::create_dir_all(demo.join("sqend")).unwrap();
    std::fs::write(demo.join(VIDEO), &video).unwrap();
    println!("sqend: {FRAMES} frames, {} machine runs, {}M instructions in {:.2}s (1536x768); identical to native Rust: {good}/{FRAMES}",
        o.host.runs, o.host.steps / 1_000_000, started.elapsed().as_secs_f64());
    if good != FRAMES { std::process::exit(1); }
}
