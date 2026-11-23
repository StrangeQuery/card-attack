//! Differences of longs, for the train: a - b (longs, thousands then the rest), as a long, held to lo..hi (cards: the
//! thousands' bounds, the rest at the low bound 0 and at the high 999). Deck: the count, lo, hi, then pairs a, b.
//! Punches each difference.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let lo = inp();
    let hi = inp();
    for _ in 0..count {
        let mut d_hi = inp();
        let mut d_lo = inp();
        let b_hi = inp();
        // the borrow first, so the thousands never pass a card on the way
        d_lo -= inp();
        if d_lo < 0 { d_lo += 999; d_lo += 1; d_hi -= 1; }
        d_hi -= b_hi;
        if d_hi < lo { d_hi = lo; d_lo = 0; }
        if d_hi > hi { d_hi = hi; d_lo = 999; }
        out(d_hi);
        out(d_lo);
    }
}
