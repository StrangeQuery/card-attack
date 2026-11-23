//! Sums, for the train, the lean kind: a long plus a constant (a long), and a second long added or taken away. Deck:
//! the count, how many longs an item has (1 or 2), the second's sign (1 or -1), the constant; then the items. Punches
//! each sum as a long (every partial sum is one).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let k = inp();
    let s2 = inp();
    let ch = inp();
    let cl = inp();
    for _ in 0..count {
        let mut hi = inp();
        let mut lo = inp();
        hi += ch;
        let over = lo - (999 - cl);
        if over > 0 { lo = over - 1; hi += 1; } else { lo = over + 999; }
        if k == 2 {
            let bh = inp();
            let bl = inp();
            if s2 > 0 {
                hi += bh;
                let over = lo - (999 - bl);
                if over > 0 { lo = over - 1; hi += 1; } else { lo = over + 999; }
            } else {
                // the borrow first, so the thousands never pass a card on the way
                lo -= bl;
                if lo < 0 { lo += 999; lo += 1; hi -= 1; }
                hi -= bh;
            }
        }
        out(hi);
        out(lo);
    }
}
