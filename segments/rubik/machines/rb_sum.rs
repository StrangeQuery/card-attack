//! Sum, many times per frame of the Rubik's cube: signed sums, for the cubies' depths and the corners of the
//! cubies' faces and stickers. Deck: groups, each the number of terms n (0 ends the deck), then n pairs: a sign
//! (-1, 0 or 1) and a value. Punches each group's sum of sign times value.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let n = inp();
        if n <= 0 { break; }
        let mut sum = 0;
        for _ in 0..n {
            let sign = inp();
            let value = inp();
            if sign < 0 { sum -= value; }
            if sign > 0 { sum += value; }
        }
        out(sum);
    }
}
