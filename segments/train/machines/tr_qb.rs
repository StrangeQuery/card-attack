//! Products, for the train, the second half: the product from its two quarter squares (longs), as a long, or over
//! 1000 rounded down (s 3). Deck: the count, s (0 or 3), then pairs of longs. Punches each as a long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let s = inp();
    for _ in 0..count {
        let mut hi = inp();
        let mut lo = inp();
        hi -= inp();
        lo -= inp();
        if lo < 0 { lo += 999; lo += 1; hi -= 1; }
        if s == 0 { out(hi); out(lo); } else if hi < 0 { out(-1); out(hi + 999 + 1); } else { out(0); out(hi); }
    }
}
