//! Over 100, for the train: a long over 100 rounded down, and the remainder. Deck: the count, then the longs (the rest
//! never below 0, so the thousands' last digits move down into the rest, the rest's into the remainder). Punches each
//! quotient (a long) and remainder (a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let hi = inp();
        let lo = inp();
        let mut q = hi / 100;
        let mut r = hi % 100;
        if r < 0 { r += 100; q -= 1; }
        out(q);
        out(r * 10 + lo / 100);
        out(lo % 100);
    }
}
