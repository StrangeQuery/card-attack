//! Over 1000, for the train: a long over 1000 rounded down, and the remainder. Deck: the count, then the longs (the rest
//! never below 0, so the thousands' last digits move down into the rest, the rest's into the remainder). Punches each
//! quotient (a long) and remainder (a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let hi = inp();
        let lo = inp();
        if hi < 0 { out(-1); out(hi + 999 + 1); } else { out(0); out(hi); }
        out(lo);
    }
}
