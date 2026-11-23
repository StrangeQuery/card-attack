//! Street edges, once per row of the street per frame of the accelerated drive, for the pixels near a line's edge:
//! how far past each edge the pixel's middle is. Deck: how many, then each: how far from the middle (64ths), the edge
//! (64ths, and 640ths over). Punches each 10 (off - l) - r (640ths), off - l kept within 50.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut gap = inp() - inp();
        if gap > 50 { gap = 50; }
        if gap < -50 { gap = -50; }
        out(10 * gap - inp());
    }
}
