//! Atans, for the swarm's angles, the second half: between two table entries (millionths of a turn, longs, at most 79
//! apart) by tenths. Deck: the count; then each: the two entries and the tenths. Punches each atan (a long).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut hi = inp();
        let mut lo = inp();
        inp();
        let mut step = inp() - lo;
        if step < 0 { step += 999; step += 1; }
        // the rest and the part pass 999 just when the rest is more than 999 less the part
        lo -= 999 - demokit::mul::times(step, inp()) / 10;
        if lo > 0 { lo -= 1; hi += 1; } else { lo += 999; }
        out(hi); out(lo);
    }
}
