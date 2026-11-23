//! Bloom's start, once per channel per frame of the accelerated drive: how far past bright each 4 x 4 block's
//! top-left pixel is. Deck: the threshold (tenths of a step: 150 of 255), how many, then each pixel's channel.
//! Punches each (v - threshold) / 10 steps, rounded down; 0 below it.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let t = inp();
    for _ in 0..inp() {
        let v = inp() - t;
        out(if v > 0 { v / 10 } else { 0 });
    }
}
