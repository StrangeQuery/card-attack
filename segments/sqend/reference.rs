//! The ending's SQ in ordinary Rust with whole numbers: the reference the host's frames are checked against. The
//! renderer (sqend.rs) is written once, against the train's operations; here each is done natively, the host does
//! each with machines on CARDIAC.
//!
//!     sqend_ref SEGMENTS_DIR        # RGB 1536x768 frames to stdout
#![allow(dead_code)]
use std::io::Write;

include!("../train/ops.rs");
include!("sqend.rs");

fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let mut o = Native::new(vec![]);
    let w = load(&mut o, &dir);
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for k in 0..FRAMES { out.write_all(&render(&mut o, &w, k)).unwrap(); }
}
