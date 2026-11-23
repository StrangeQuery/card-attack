//! Negatives, for the deep's walks back (the host negated them): a row's ray slopes taken the other way. Deck:
//! how many, then the cards. Punches each card's negative.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let n = inp();
    for _ in 0..n {
        let x = inp();
        out(0 - x);
    }
}
