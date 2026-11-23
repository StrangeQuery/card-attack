//! Terms, for the swarm's sums: a number of one card or of three (millions, thousands, rest; the last two 0..999),
//! as three cards, taken away (its negative) when its sign is set. Deck: the count; then each: the sign (1 to take it
//! away, else 0), how many cards (1 or 3) and the cards. Punches each as three cards.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let away = inp();
        let (mut h2, mut h1, mut h0) = (0, 0, 0);
        if inp() == 1 { h0 = inp(); if h0 < 0 { h2 = -1; h1 = 999; h0 += 999; h0 += 1; } } else { h2 = inp(); h1 = inp(); h0 = inp(); }
        if away > 0 {
            // each card's complement, the borrow running up from the rest
            h2 = -h2;
            if h0 > 0 { h0 = 999 - h0 + 1; h1 = 999 - h1; h2 -= 1; } else if h1 > 0 { h1 = 999 - h1 + 1; h2 -= 1; }
        }
        out(h2); out(h1); out(h0);
    }
}
