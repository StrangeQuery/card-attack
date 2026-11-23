//! Glow tables, once per amount per channel the accelerated drive's lights need (each worked out once): the glow's
//! profile times the amount. Deck: the amount (tenths of a step), how many, then each the profile (500ths, glowt's).
//! Punches each profile amount / 500, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let amount = inp();
    for _ in 0..inp() {
        let g = inp();
        let (mut times, mut each) = (g, amount);
        if amount < g { times = amount; each = g; }
        let mut q = 0;
        let mut sum = -500;
        for _ in 0..times {
            sum += each;
            while sum >= 0 { sum -= 500; q += 1; }
        }
        out(q);
    }
}
