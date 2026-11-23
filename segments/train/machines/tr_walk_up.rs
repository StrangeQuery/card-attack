//! Walks, for the train, adding: a value carried across n places (n under 999: an edge's rows, a span's pixels), each
//! place the change's whole step + one more each time the remainders carry, so the k-th place has the start's value
//! + the change's size times (k + 1) / (n + 1), rounded down. Deck: the count; then each walk: n; the start's value
//! (a long); the change's whole step over n + 1 (a long) and its remainder (a card); and n + 1. Punches each place's
//! value, a long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let n = inp();
        let mut v_hi = inp();
        let mut v_lo = inp();
        let w_hi = inp();
        let w_lo = inp();
        let rem = inp();
        let steps = inp();
        // the remainders so far less n + 1: below 0 until they carry one
        let mut over_n = -steps;
        for _ in 0..n {
            v_hi += w_hi;
            let over = v_lo - (999 - w_lo);
            if over > 0 { v_lo = over - 1; v_hi += 1; } else { v_lo = over + 999; }
            over_n += rem;
            if over_n >= 0 {
                over_n -= steps;
                if v_lo == 999 { v_lo = 0; v_hi += 1; } else { v_lo += 1; }
            }
            out(v_hi);
            out(v_lo);
        }
    }
}
