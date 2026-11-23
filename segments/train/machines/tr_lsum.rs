//! Light's sum, for the train: a + s + e (cards 0..999) as the card it gives (at most 999) and how far it passes 999
//! (at most 999). Deck: the count, then items a, s, e. Punches the two for each.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut lo = inp();
        let mut hi = 0;
        for _ in 0..2 {
            let x = inp();
            let over = lo - (999 - x);
            if over > 0 { lo = over - 1; hi += 1; } else { lo = over + 999; }
        }
        if hi == 0 { out(lo); out(0); } else { out(999); if hi == 1 && lo < 999 { out(lo + 1); } else { out(999); } }
    }
}
