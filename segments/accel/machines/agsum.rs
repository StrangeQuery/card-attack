//! Glows laid on, as many pixels as the host has, per frame of the accelerated drive, a channel at a time. Deck: how
//! many, then each: the pixel's channel (tenths of a step), how many glows reach it, then each glow's (tenths of a
//! step, 0 or more). Punches each the sum, at most 999.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut v = inp();
        for _ in 0..inp() {
            let g = inp();
            // (at most 999: past it, the sum would pass a card)
            if g > 999 - v { v = 999; } else { v += g; }
        }
        out(v);
    }
}
