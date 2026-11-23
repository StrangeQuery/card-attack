//! Running sums, for the train. Deck: the count of lists; then each list: its length (a long) and its values (cards).
//! Punches each value's sum with those before it in its list, a long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut n_hi = inp();
        let mut n_lo = inp();
        let mut hi = 0;
        let mut lo = 0;
        loop {
            if n_hi == 0 && n_lo == 0 { break; }
            n_lo -= 1;
            if n_lo < 0 { n_lo = 999; n_hi -= 1; }
            let x = inp();
            if x >= 0 {
                let over = lo - (999 - x);
                if over > 0 { lo = over - 1; hi += 1; } else { lo = over + 999; }
            } else {
                lo += x;
                if lo < 0 { lo += 999; lo += 1; hi -= 1; }
            }
            out(hi);
            out(lo);
        }
    }
}
