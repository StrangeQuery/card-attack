//! At or past, as many as the host has, per frame of the accelerated drive (a depth test: a pixel's distance against
//! a thing's). Deck: how many, then each a and b. Punches each 1 if a is b or more, else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let a = inp();
        out(if a >= inp() { 1 } else { 0 });
    }
}
