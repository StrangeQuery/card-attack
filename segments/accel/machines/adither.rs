//! Dither, once per half row and channel per frame of the accelerated drive: 16-bit colour. Deck: the channel's
//! most (31 for red and blue, 63 for green), then for each of the 384 pixels its channel (tenths of a step) and its
//! threshold (tenths of a step: the 4 x 4 ordered dither's). Punches each (v + t) / 10, 0 to the most.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let most = inp();
    for _ in 0..384 {
        let mut level = (inp() + inp()) / 10;
        if level < 0 { level = 0; }
        if level > most { level = most; }
        out(level);
    }
}
