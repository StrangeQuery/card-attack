//! Spark streaks' places, per spark per axis per frame of the accelerated drive while sparks fly: a line from its
//! head to its tail. Deck: the head's pixel p, the way to the tail d, n (the line's places less one, at least 1).
//! Punches for each place j from 0 to n p + d j / n, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let p = inp();
    let d = inp();
    let n = inp();
    let a = abs(d);
    let mut q = 0;
    let mut r = 0;
    for j in 0..n + 1 {
        if j > 0 {
            r += a;
            while r >= n { r -= n; q += 1; }
        }
        out(if d < 0 { p - q - if r > 0 { 1 } else { 0 } } else { p + q });
    }
}
