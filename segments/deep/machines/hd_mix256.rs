//! Mix in 256ths, for the dive's creatures at 1536x768 (the jellyfish's halos). Deck: how many, then for each a, b
//! (0..255) and k (0..255). Punches, per one, a + (b - a) k / 256, toward zero: the difference added k times, the
//! 256s carried out as they fill.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let a = inp();
        let b = inp();
        let k = inp();
        let size = abs(b - a);
        let mut part = 0;
        let mut whole = 0;
        for _ in 0..k {
            part += size;
            if part >= 256 { part -= 256; whole += 1; }
        }
        if b < a { whole = -whole; }
        out(a + whole);
    }
}
