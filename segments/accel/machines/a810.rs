//! Eight tenths, as many as the host has, per frame of the accelerated drive (the post's arm, a little darker). Deck:
//! how many, then each a channel (tenths of a step, at most 124). Punches each 8 v / 10, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        out(8 * inp() / 10);
    }
}
