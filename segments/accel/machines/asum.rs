//! Sums, as many as the host has, per frame of the accelerated drive. Deck: how many, then each a, b and 1 or -1.
//! Punches each a + b, or a - b.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let a = inp();
        let b = inp();
        out(if inp() < 0 { a - b } else { a + b });
    }
}
