//! Products from quarter squares, for the swarm's arithmetic: a b is the difference of the two quarter squares the
//! host looked up (longs: thousands, rest). Deck: the count; then each product: its place (0 as it is, 1 times 1000,
//! 2 times a million, for a product under 1000 in size) and the two quarter squares. Punches each as three cards,
//! millions, thousands and rest, the last two 0..999.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let place = inp();
        let mut th = inp();
        let mut rest = inp();
        th -= inp();
        rest -= inp();
        if rest < 0 { rest += 999; rest += 1; th -= 1; }
        if place == 0 {
            if th < 0 { out(-1); out(th + 999 + 1); } else { out(0); out(th); }
            out(rest);
        } else if place == 1 {
            out(th); out(rest); out(0);
        } else {
            if th < 0 { out(rest - 999 - 1); } else { out(rest); }
            out(0); out(0);
        }
    }
}
