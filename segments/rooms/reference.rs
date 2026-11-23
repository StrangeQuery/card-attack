//! The rooms (the dorm, the screening room), drawn in ordinary Rust with whole numbers: the reference the host's
//! frames are checked against. The renderer (rooms.rs) is written once, against the train's operations
//! (segments/train/ops.rs): here each operation is done natively, as its definition says; the host does each with
//! machines on CARDIAC.
//!
//!     rooms_ref SEGMENTS_DIR [FRAMES]        # FRAMES: k,k,... (default all); RGB 1536x768 frames to stdout
//!     ROOMS_COUNT=1 rooms_ref ...            # the operations' counts
#![allow(clippy::needless_range_loop, clippy::too_many_arguments, dead_code)]
use std::io::Write;

include!("../train/ops.rs");
include!("rooms.rs");

/// Frames a..a+n of a video of frames of `size` bytes.
fn frames(path: &str, size: usize, a: i64, n: i64) -> Vec<u8> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    f.seek(SeekFrom::Start(a as u64 * size as u64)).unwrap();
    let mut b = vec![0u8; n as usize * size];
    f.read_exact(&mut b).unwrap();
    b
}

pub fn srcs(dir: &str) -> Srcs {
    let fr = 1536 * 768 * 3;
    Srcs {
        modex: frames(&format!("{dir}/modex/video_hd.rgb"), fr, MODEX0, MODEX_N),
        polys: frames(&format!("{dir}/polys/video.rgb"), 192 * 96 * 3, POLYS0, POLYS_N),
        drive: frames(&format!("{dir}/accel/video_hd.rgb"), fr, DRIVE0, DRIVE_N),
        dive: frames(&format!("{dir}/deep/video_hd.rgb"), fr, 0, DIVE_N),
        brag: std::fs::read(format!("{dir}/ui/accelbrag.rgba")).unwrap(),
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = &a[1];
    let mut o = Native::new(vec![]);
    let w = load(dir);
    let s = srcs(dir);
    let only: Option<Vec<i64>> = a.get(2).map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for k in 0..FRAMES {
        if let Some(l) = &only { if !l.contains(&k) { continue; } }
        out.write_all(&render(&mut o, &w, &s, k)).unwrap();
        eprintln!("frame {k}");
    }
    if std::env::var("ROOMS_COUNT").is_ok() { for (op, c) in &o.count { eprintln!("{op:12} {c:>12}"); } }
}
