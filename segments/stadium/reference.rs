//! The stadium party (the film's last segment), drawn in ordinary Rust with whole numbers: the reference the host's
//! frames are checked against. The renderer (frame.rs and the files it includes) is written once, against the train's
//! operations (segments/train/ops.rs): here each operation is done natively, as its definition says; the host does
//! each with machines on CARDIAC. See README.md.
//!
//!     stadium_ref SEGMENTS_DIR [FRAMES]      # FRAMES: k,k,... (default all); RGB 1536x768 frames to stdout
#![allow(clippy::needless_range_loop, clippy::too_many_arguments, dead_code)]
use std::io::Write;

include!("../train/ops.rs");
include!("scene.rs");
include!("maths.rs");
include!("clock.rs");
include!("raster.rs");
include!("light.rs");
include!("pictures.rs");
include!("crowd.rs");
include!("shade.rs");
include!("glow.rs");
include!("beams.rs");
include!("air.rs");
include!("things.rs");
include!("frame.rs");
include!("party.rs");
include!("inputs.rs");

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = &a[1];
    let only: Option<Vec<i64>> = a.get(2).map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let mut o = Native::new(read_trig(dir));
    let w = load(&mut o, dir);
    // the party version (night/video_hd.rgb in, the world at half size) or with STADIUM_FULL the full one
    // (night/video_full.rgb)
    let hd = std::env::var("STADIUM_FULL").is_err();
    let night = std::fs::read(format!("{dir}/night/{}", if hd { "video_hd.rgb" } else { "video_full.rgb" })).ok();
    let cams = night_cams(dir);
    let night_part = night.as_ref().map(|n| night_dissolve_frames(n, &cams));
    let inp = Inputs { night: night_part.as_deref(), night_cam: &cams, hd };
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for k in 0..FRAMES {
        if let Some(l) = &only { if !l.contains(&k) { continue; } }
        out.write_all(&render(&mut o, &w, &inp, k)).unwrap();
        eprintln!("frame {k}");
    }
    if std::env::var("STADIUM_COUNT").is_ok() {
        for ((s, op), c) in &o.by_stage { eprintln!("{s:10} {op:12} {c:>12}"); }
    }
}
