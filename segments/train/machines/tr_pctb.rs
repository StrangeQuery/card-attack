//! Shares and mixes, for the train, the second half: the product from its two quarter squares (longs), over 100 cut
//! toward zero, added to a. Deck: the count, then each: the two quarter squares and a. Punches each result (a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut hi = inp();
        let mut lo = inp();
        hi -= inp();
        lo -= inp();
        if lo < 0 { lo += 999; lo += 1; hi -= 1; }
        let a = inp();
        // p / 100 toward zero: p = hi 1000 + lo; below 0 one more back unless it divides
        let mut q = hi * 10 + lo / 100;
        if hi < 0 && lo % 100 != 0 { q += 1; }
        out(a + q);
    }
}
