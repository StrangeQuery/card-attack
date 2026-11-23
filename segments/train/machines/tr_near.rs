//! The nearest piece, for the train's depth test: of each group of depths (K / z, longs), the place of the
//! largest, the first of equals. Deck: the count; then each group: how many, and the depths. Punches each place.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let n = inp();
        let mut best_hi = inp();
        let mut best_lo = inp();
        let mut best = 0;
        for k in 1..n {
            let hi = inp();
            let lo = inp();
            let d = hi - best_hi;
            if d > 0 || (d == 0 && lo > best_lo) { best_hi = hi; best_lo = lo; best = k; }
        }
        out(best);
    }
}
