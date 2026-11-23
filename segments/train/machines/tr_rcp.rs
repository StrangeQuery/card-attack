//! Depths to distances, for the train, the first step: a depth K / z (a long, 1 or more) as its first four digits, for
//! the host to look up K / 0.7 over them, and how many places were dropped. Deck: the count, then the depths. Punches
//! the four digits (a long) and the places (0, 1 or 2).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let hi = inp();
        let lo = inp();
        if hi < 10 {
            out(hi); out(lo); out(0);
        } else if hi < 100 {
            // over 10: hi 100 + lo / 10, as a long
            out(hi / 10); out((hi % 10) * 100 + lo / 10); out(1);
        } else {
            out(hi / 100); out((hi % 100) * 10 + lo / 100); out(2);
        }
    }
}
