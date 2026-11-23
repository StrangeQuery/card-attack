//! Quarter squares, the first half of a product, for the swarm's arithmetic: a b is (a + b)^2 / 4 - (a - b)^2 / 4,
//! each rounded down (a + b and a - b are both even or both odd, so nothing is lost). Deck: the count, then pairs a,
//! b (cards). Punches for each the sizes of a + b and of a - b, each as two cards, thousands (0 or 1) and the rest,
//! for the host to look up in the printed table of quarter squares.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        // the sum of the sizes may pass a card: over is how far past 999 it goes
        let (x, y) = (abs(a), abs(b));
        let over = x - (999 - y);
        let (mut big_hi, mut big_lo) = (0, over + 999);
        if over > 0 { big_hi = 1; big_lo = over - 1; }
        let small = abs(x - y);
        if (a < 0) == (b < 0) {
            out(big_hi); out(big_lo); out(0); out(small);
        } else {
            out(0); out(small); out(big_hi); out(big_lo);
        }
    }
}
