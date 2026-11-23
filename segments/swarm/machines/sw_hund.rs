//! Hundreds, for the swarm's square roots: a number of three cards (millions, thousands, rest; under ten million) as
//! its hundreds (a long) and the rest (0..99). Deck: the count, then the numbers. Punches each's hundreds and rest.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let millions = inp();
        let thousands = inp();
        let rest = inp();
        let (t, r) = (digit(thousands, 0), digit(rest, 0));
        out(millions * 10 + t);
        out((thousands - t * 100) * 10 + r);
        out(rest - r * 100);
    }
}
