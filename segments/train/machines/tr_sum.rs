//! Sums, for the train, of longs (thousands, then the rest 0..999), each added or taken away. Deck: the count, how
//! many terms; then each sum's terms: 1 to add it or -1 to take it away, and the long. Punches each sum as a long. The
//! rests are added a card at a time, carrying a thousand.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let terms = inp();
    for _ in 0..count {
        let mut hi = 0;
        let mut lo = 0;
        for _ in 0..terms {
            let sign = inp();
            let t_hi = inp();
            let t_lo = inp();
            if sign > 0 {
                hi += t_hi;
                // the rests' sum passes 999 just when one is more than 999 less the other
                let over = lo - (999 - t_lo);
                if over > 0 { lo = over - 1; hi += 1; } else { lo = over + 999; }
            } else {
                // the borrow first, so the thousands never pass a card on the way
                lo -= t_lo;
                if lo < 0 { lo += 999; lo += 1; hi -= 1; }
                hi -= t_hi;
            }
        }
        out(hi);
        out(lo);
    }
}
