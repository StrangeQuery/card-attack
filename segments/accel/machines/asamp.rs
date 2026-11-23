//! Samples, as many as the host has, per frame of the accelerated drive: which of a picture's pixels each of a box's
//! shows (the nearest). Deck: how many, then each: the box's size n and the picture's p. Punches each u p / n for
//! u from 0 to n - 1, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let n = inp();
        let p = inp();
        let mut at = 0;
        let mut sum = 0;
        for _ in 0..n {
            out(at);
            sum += p;
            while sum >= n { sum -= n; at += 1; }
        }
    }
}
