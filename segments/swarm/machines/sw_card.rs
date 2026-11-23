//! Cards, for the swarm: a number of three cards (millions, thousands, rest) that fits one card, as that card.
//! Deck: the count, then the numbers. Punches each as one card.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        inp();
        let thousands = inp();
        let rest = inp();
        // SAFETY: it fits a card, so its thousands are 0, or 999 below 0
        unsafe { assume(thousands == 0 || thousands == 999); }
        if thousands > 0 { out(rest - 999 - 1); } else { out(rest); }
    }
}
