//! Long sums, for the swarm's arithmetic. Deck: the count; then each sum: how many terms, and each term as three
//! cards, millions, thousands and rest (the last two 0..999). Punches each sum the same way. Two cards' sum passes
//! 999 just when the first is more than 999 less the second: a card at a time, carrying.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let terms = inp();
        let (mut h2, mut h1, mut h0) = (0, 0, 0);
        for _ in 0..terms {
            h2 += inp();
            let t1 = inp();
            let over = h0 - (999 - inp());
            if over > 0 { h0 = over - 1; if h1 == 999 { h1 = 0; h2 += 1; } else { h1 += 1; } } else { h0 = over + 999; }
            let over = h1 - (999 - t1);
            if over > 0 { h1 = over - 1; h2 += 1; } else { h1 = over + 999; }
        }
        // the millions as a card keeps them: a long number is nine digits, and a negative one wraps (998 999 999 is
        // -1000001), so a sum's thousands of millions are dropped (1000 taken off as 999 and 1: a card holds no 1000)
        while h2 > 999 { h2 -= 999; h2 -= 1; }
        out(h2); out(h1); out(h0);
    }
}
