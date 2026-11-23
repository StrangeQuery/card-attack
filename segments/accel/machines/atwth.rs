//! Picture sizes, as many as the host has, per frame of the accelerated drive. Deck: how many, then each a size (a
//! pixel and 16ths). Punches each (16 p + f + 8) / 16 + 1: the picture's pixels across it (p + 1, and 1 more for f of
//! 8 or more); and 1 if p is at least 1 (a car that small is drawn), else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let p = inp();
        out(if inp() >= 8 { p + 2 } else { p + 1 });
        out(if p > 0 { 1 } else { 0 });
    }
}
