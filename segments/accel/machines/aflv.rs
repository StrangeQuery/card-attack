//! Flare colours' indexes, as many as the host has, per frame of the accelerated drive: a colour as tenths of a step
//! and where the falloff reaches it. Deck: how many, then each: the colour (8-bit) and the channel's steps (tenths,
//! SCALE). Punches each v - 1, v = colour SCALE / 255 rounded down (the place of its index in LN), or -1 for v of 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let x = inp();
        let scale = inp();
        let mut v = 0;
        let mut sum = 0;
        for _ in 0..x {
            sum += scale;
            while sum >= 255 { sum -= 255; v += 1; }
        }
        out(if v > 0 { v - 1 } else { -1 });
    }
}
