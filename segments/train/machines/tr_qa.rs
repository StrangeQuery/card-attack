//! Products, for the train, the first half: a b is (a + b)^2 / 4 - (a - b)^2 / 4, both rounded down (nothing lost),
//! the two the other way round when the signs differ (so the first less the second is the product with its sign).
//! Deck: the count, then pairs a, b (cards). Punches the two sizes to look up in the table of quarter squares, each
//! as its thousands (0 or 1) and the rest.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let x = abs(a);
        let y = abs(b);
        // the sizes' sum may pass a card: how far past 999
        let over = x - (999 - y);
        let small = abs(x - y);
        if (a < 0) == (b < 0) {
            if over > 0 { out(1); out(over - 1); } else { out(0); out(over + 999); }
            out(0); out(small);
        } else {
            out(0); out(small);
            if over > 0 { out(1); out(over - 1); } else { out(0); out(over + 999); }
        }
    }
}
