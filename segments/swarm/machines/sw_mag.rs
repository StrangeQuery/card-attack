//! Signs and sizes, for the swarm's arithmetic: a number of one card, or of three (millions, thousands, rest; the
//! last two 0..999), as its sign and its size's three cards (each 0..999). Deck: the count; then each: how many cards
//! (1 or 3) and the cards. Punches for each 1 if it is below 0 (else 0), then its size.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let (mut h2, mut h1, mut h0) = (0, 0, 0);
        if inp() == 1 { h0 = inp(); if h0 < 0 { h2 = -1; h1 = 999; h0 += 999; h0 += 1; } } else { h2 = inp(); h1 = inp(); h0 = inp(); }
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
