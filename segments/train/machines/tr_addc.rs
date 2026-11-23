//! Sums of cards, for the train: each term added or taken away, and a constant. Deck: the count, the constant, how
//! many terms, each one's sign (1 or -1, at most 6); then each sum's terms. Punches each sum as two cards, its
//! thousands and the rest, both with the sum's sign (-1234 is -1, -234): at most 6,993 in size, so never cut short.
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
        // the thousands toward zero, and the rest with the sum's sign
        let mut th = 0;
        while s > 999 { s -= 999; s -= 1; th += 1; }
        while s < -999 { s += 999; s += 1; th -= 1; }
        out(th);
        out(s);
    }
}
