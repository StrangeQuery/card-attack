//! Sums of longs, for the swarm's arithmetic. Deck: the count, the longs to a sum; then each sum's longs (thousands,
//! rest 0..999). Punches each sum as a long: the rests' sum passes 999 just when the first is more than 999 less the
//! second.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let terms = inp();
    for _ in 0..count {
        let (mut hi, mut lo) = (0, 0);
        for _ in 0..terms {
            hi += inp();
            lo -= 999 - inp();
            if lo > 0 { lo -= 1; hi += 1; } else { lo += 999; }
        }
        out(hi); out(lo);
    }
}
