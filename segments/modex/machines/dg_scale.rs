//! Scale, the degauss's arithmetic on a few numbers at a time: per frame its sizes (the jolt's, its spreads, the
//! swirl's strength, the flash, the blotch's strength); per row and column of the jolt its waves; per pixel of the
//! blotch its tints; per light and per swirl in a frame of the jolt. Deck: how many; then each: base, a (0 or
//! more), b, c (1 or more). Punches each base + a b / c, the product's part rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let base = inp();
        let times = inp();
        let b = inp();
        let c = inp();
        // a |b| / c without the product, which would pass a card: |b| is c whole times and a rest; the wholes added
        // a times, and the rest a times, a c carried out whenever the sum would reach it (so it stays under c)
        let (whole, rest) = divide(abs(b), c);
        let gap = c - rest;
        let mut part = 0;
        let mut sum = 0;
        for _ in 0..times {
            part += whole;
            if sum >= gap { sum -= gap; part += 1; } else { sum += rest; }
        }
        // below zero, a part left over rounds down one more
        if b < 0 {
            if sum > 0 { part += 1; }
            out(base - part);
        } else {
            out(base + part);
        }
    }
}
