//! Means of four, as many as the host has, per frame of the accelerated drive, a channel at a time (our car's pixel
//! from its four samples at twice the size). Deck: how many, then each its four (0 or more). Punches each ((a + b) /
//! 2 + (c + d) / 2) / 2, each half rounded down (the halves taken first: a sum could pass a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut first = 0;
        let mut second = 0;
        for k in 0..3 {
            let (a, b) = if k < 2 { (inp(), inp()) } else { (first, second) };
            let (ha, hb) = (a / 2, b / 2);
            let mut v = ha + hb;
            if a - ha - ha == 1 && b - hb - hb == 1 { v += 1; }
            if k == 0 { first = v; } else if k == 1 { second = v; } else { out(v); }
        }
    }
}
