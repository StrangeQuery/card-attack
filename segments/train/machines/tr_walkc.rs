//! Walks by slopes over cards, for the train's rays and normals: a card carried across n places (n under 1000), each
//! step the slope's whole part (a card) and one more each time its thousandths carry (from f0), so the k-th place has
//! the start's value + fl(f0 + a k, 1000). Deck: the count; then each walk: n, the start's value, the slope's whole
//! part and its thousandths, f0. Punches each place's value.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let n = inp();
        let mut v = inp();
        let w = inp();
        let frac = inp();
        let mut acc = inp();
        for _ in 0..n {
            out(v);
            v += w;
            let over = acc - (999 - frac);
            if over > 0 { acc = over - 1; v += 1; } else { acc = over + 999; }
        }
    }
}
