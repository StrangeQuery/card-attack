//! The tone curve, for the train, the first half: a channel's light, its card, 25 times the part past 999 in tenths
//! (the host's printed table, a long) and the light the lens adds (a long), as one index for the curve's table (under
//! 40,000). Deck: the count, then items: the card, the two longs. Punches each index (a long).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut r = inp();
        let mut hi = 0;
        for _ in 0..2 {
            hi += inp();
            let x = inp();
            let over = r - (999 - x);
            if over > 0 { r = over - 1; hi += 1; } else { r = over + 999; }
        }
        out(hi);
        out(r);
    }
}
