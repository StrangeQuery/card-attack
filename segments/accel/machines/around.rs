//! Round, as many as the host has, per frame of the accelerated drive: places taken round a size (the dash's 480).
//! Deck: the size, how many, then each place (0 or more). Punches each less the size as often as it fits.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let size = inp();
    let count = inp();
    for _ in 0..count {
        let mut v = inp();
        while v >= size { v -= size; }
        out(v);
    }
}
