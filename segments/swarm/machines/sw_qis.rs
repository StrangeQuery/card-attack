//! Quarter squares for signed products, for the swarm's sums: a b is (a + b)^2 / 4 - (a - b)^2 / 4 for sizes a and
//! b; the product taken away (the two quarter squares the other way round) when an odd number of three signs is set
//! (the term's own and its factors'). Deck: the count, then each: a, b (sizes, 0..999) and the three signs (0 or 1).
//! Punches for each the two numbers to look up in the printed table of quarter squares, the first less the second
//! the product: each as two cards, thousands (0 or 1) and the rest.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let odd = inp() + inp() + inp();
        // the sum may pass a card: over is how far past 999 it goes
        let over = a - (999 - b);
        let (mut big_hi, mut big_lo) = (0, over + 999);
        if over > 0 { big_hi = 1; big_lo = over - 1; }
        let small = abs(a - b);
        if odd == 1 || odd == 3 {
            out(0); out(small); out(big_hi); out(big_lo);
        } else {
            out(big_hi); out(big_lo); out(0); out(small);
        }
    }
}
