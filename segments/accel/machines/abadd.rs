//! Bloom laid on, per 4 x 4 block that has some per channel per frame of the accelerated drive. Deck: how many blocks,
//! then each: its amount (tenths of a step), how many of its pixels, then their channels. Punches each pixel's
//! channel and the amount, at most 999.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let a = inp();
        for _ in 0..inp() {
            let v = inp();
            out(if a > 999 - v { 999 } else { v + a });
        }
    }
}
