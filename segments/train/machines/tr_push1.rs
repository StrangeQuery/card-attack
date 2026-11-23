//! The tone curve, for the train, a little more colour, the first half: for each of a pixel's three toned channels
//! (0..255), 3 t - s (s their sum), for the host's table of twentieths. Deck: the count, then each pixel's three.
//! Punches the three.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let c = inp();
        let s = a + b + c;
        out(a + a + a - s);
        out(b + b + b - s);
        out(c + c + c - s);
    }
}
