//! Flare streak heights, per row of a flare per frame of the accelerated drive. Deck: how many, then each g (aflax's).
//! Punches each g g / 32 rounded down, at most 192.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let g = inp();
        let mut q = 0;
        let mut rest = 0;
        let mut k = g;
        while k > 0 && q < 192 {
            rest += g;
            while rest >= 32 { rest -= 32; q += 1; }
            k -= 1;
        }
        out(if q > 192 { 192 } else { q });
    }
}
