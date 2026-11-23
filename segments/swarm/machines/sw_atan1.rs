//! Atans, for the swarm's angles, the first half: a ratio (hundred-thousandths, 0..100000, a long) as its place in
//! the printed table of atan every 2000th (each entry printed with the next). Deck: the count, then the ratios.
//! Punches for each the entry below (0..2000, a long) and the tenths on to the next (0..9): the entry is the ratio
//! over 50, twenty a thousand and the rest's fifties, and the tenths the rest of that's fives.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let hi = inp();
        let mut rest = inp();
        // SAFETY: the ratio is 0..100000, so its thousands are 0..100
        unsafe { assume(hi >= 0 && hi <= 100); }
        if hi == 100 { out(2); out(0); out(0); continue; }
        let (mut thousands, mut at) = (0, hi * 10);
        if at > 499 { at -= 500; thousands = 1; }
        at *= 2;
        if at > 999 { at -= 999; at -= 1; thousands += 1; }
        while rest >= 50 { rest -= 50; at += 1; }
        out(thousands); out(at);
        out(rest / 5);
    }
}
