//! Thousandths, for the swarm's arithmetic: a number of three cards (millions, thousands, rest) over 1000, rounded
//! down: its millions and thousands moved down a card. Deck: the count, then the numbers. Punches each as three cards.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let h2 = inp();
        let h1 = inp();
        inp();
        if h2 < 0 { out(-1); out(h2 + 999 + 1); } else { out(0); out(h2); }
        out(h1);
    }
}
