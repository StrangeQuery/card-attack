//! Glow made four times as fine, for the train: along a line of values (cards 0..999), between each and the next the
//! four values a, a + q, a + h, a + h + q, h half the change and q half of h (each cut toward zero). Deck: the count of
//! lines, then each line: its count of values, the values. Punches the four values for each step along each line.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let n = inp();
        let mut a = inp();
        for _ in 1..n {
            let b = inp();
            let d = b - a;
            let h = d / 2;
            let q = h / 2;
            out(a);
            out(a + q);
            out(a + h);
            out(a + h + q);
            a = b;
        }
    }
}
