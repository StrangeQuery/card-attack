//! Spark streaks' amounts, per spark per channel and weight per frame of the accelerated drive while sparks fly: how
//! much each place of the line takes, fading to 2 / 5 at the tail. Deck: the amount, n (the line's places less one).
//! Punches for each place j from 0 to n the amount less 3 amount j / (5 n), rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let amount = inp();
    let n = inp();
    // 3 amount j / (5 n) as (3 amount j / 5) / n, so that 5 n (past a card for a line of 200 places or more) is never
    // needed: amount is 5 fifths + ones, the ones' fifths are counted in f, and r counts toward the next n
    let mut fifths = 0;
    let mut ones = amount;
    while ones >= 5 { ones -= 5; fifths += 1; }
    let mut q = 0;
    let mut r = -n;
    let mut f = -5;
    for j in 0..n + 1 {
        if j > 0 {
            for _ in 0..3 {
                f += ones;
                r += fifths;
                if f >= 0 { f -= 5; r += 1; }
            }
            while r >= 0 { r -= n; q += 1; }
        }
        out(amount - q);
    }
}
