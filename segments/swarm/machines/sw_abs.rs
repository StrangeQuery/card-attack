//! Sizes, for the swarm's long division: a number of three cards (millions, thousands, rest; the last two 0..999) as
//! its sign and its size. Deck: the count, then the numbers. Punches for each 1 if it is below 0 (else 0), then its
//! size's three cards.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let (mut h2, mut h1, mut h0) = (inp(), inp(), inp());
        if h2 < 0 {
            // each card's complement, the borrow running up from the rest
            h2 = -h2;
            if h0 > 0 { h0 = 999 - h0 + 1; h1 = 999 - h1; h2 -= 1; } else if h1 > 0 { h1 = 999 - h1 + 1; h2 -= 1; }
            out(1);
        } else {
            out(0);
        }
        out(h2); out(h1); out(h0);
    }
}
