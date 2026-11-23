//! Products of a card and a long, for the train: the two parts' products (the long's thousands times the card, and
//! its rest times the card: longs) put together, p 1000 + q, as three cards (millions, thousands, rest). Deck: the
//! count, then the pairs. Punches each as three cards.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut m = inp();
        let mut th = inp();
        let q_hi = inp();
        let r = inp();
        // q's thousands carried into p's rest (or borrowed from it)
        if q_hi >= 0 {
            let over = th - (999 - q_hi);
            if over > 0 { th = over - 1; m += 1; } else { th = over + 999; }
        } else {
            th += q_hi;
            if th < 0 { th += 999; th += 1; m -= 1; }
        }
        out(m);
        out(th);
        out(r);
    }
}
