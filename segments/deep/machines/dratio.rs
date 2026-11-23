//! Ratio, once per screen row per frame of the dive while the portal shows. Deck: how many, then pairs of a larger
//! and a smaller size. Punches, per pair, 32 times the smaller over the larger, rounded down (0..32; 0 when both are
//! 0): 32 when they are the same, else binary long division, five doublings of the remainder.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let large = inp();
        let mut small = inp();
        let mut ratio = 0;
        for _ in 0..5 {
            small += small;
            ratio += ratio;
            if small >= large && large > 0 { small -= large; ratio += 1; }
        }
        if small == large && large > 0 { ratio = 32; }   // the same size: the remainder is still all of it
        out(ratio);
    }
}
