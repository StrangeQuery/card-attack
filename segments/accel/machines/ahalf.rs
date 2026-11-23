//! Halves, as many as the host has, per frame of the accelerated drive: a size of pixels and 16ths halved. Deck: how
//! many, then each the pixels (0 or more) and 16ths. Punches each the half, 8 p + f / 2 16ths (rounded down), as
//! pixels and 16ths.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let p = inp();
        let half = p / 2;
        let mut f = inp() / 2;
        if half + half < p { f += 8; }
        out(half);
        out(f);
    }
}
