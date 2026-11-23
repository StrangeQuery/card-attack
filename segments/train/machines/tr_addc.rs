//! Sums of cards, for the train: each term added or taken away, and a constant. Deck: the count, the constant, how
//! many terms, each one's sign (1 or -1, at most 6); then each sum's terms. Punches each sum (a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let c = inp();
    let terms = inp();
    let mut signs = [0i16; 6];
    for i in 0..6 { if (i as i16) < terms { signs[i] = inp(); } }
    for _ in 0..count {
        let mut s = c;
        for i in 0..6 {
            if (i as i16) >= terms { break; }
            if signs[i] > 0 { s += inp(); } else { s -= inp(); }
        }
        out(s);
    }
}
