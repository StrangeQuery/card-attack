//! Quarters, for the swarm's angles: an atan (millionths of a turn, a long under a quarter turn) turned to its point's
//! quarter: C +- it, taken away when an odd number of the flags (y the larger, x below 0, y below 0) are set; C a
//! quarter turn when y is the larger (three when y is also below 0), else a half turn when x is below 0. Deck: the
//! count; then each: the atan and the four flags (the last: both 0). Punches each angle (a long, 0..999999).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut hi = inp();
        let mut lo = inp();
        let swap = inp();
        let xneg = inp();
        let yneg = inp();
        let odd = swap + xneg + yneg;
        if odd == 1 || odd == 3 { hi = -hi; if lo > 0 { hi -= 1; lo = 999 - lo + 1; } }
        if swap > 0 { hi += 250; if yneg > 0 { hi += 500; } } else if xneg > 0 { hi += 500; }
        if hi < 0 { hi += 999; hi += 1; }
        if inp() > 0 { hi = 0; lo = 0; }
        out(hi); out(lo);
    }
}
