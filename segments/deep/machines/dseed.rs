//! Seed, once for the whole dive: the marine snow. Deck: how many specks, and a seed (0..999). Punches, per speck,
//! its place in a box 40 units across, 14 up and 14 along, in tenths: across -200..199, up 0..139, along 0..139,
//! from three numbers of the random sequence.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let mut seed = inp();
    for _ in 0..count {
        // 4 seed / 10 and 14 seed / 100, a digit at a time so no product passes a card
        seed = lcg::next(seed);
        out(seed / 10 * 4 + seed % 10 * 4 / 10 - 200);
        for _ in 0..2 {
            seed = lcg::next(seed);
            out(seed / 100 * 14 + seed % 100 * 7 / 50);
        }
    }
}
