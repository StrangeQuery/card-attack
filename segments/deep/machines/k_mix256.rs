//! Mix in 256ths, for the dive's ending at 1536x768 (the dial's glow on the water, the silt). Deck: how many, then for
//! each a, b (0..255) and k (0..255). Punches, per one, a + (b - a) k / 256, toward zero: the difference's size and k,
//! the larger added as many times as the smaller, the 256s carried out (hd_mix256's, in fewer steps).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let a = inp();
        let b = inp();
        let k = inp();
        let size = abs(b - a);
        let (mut times, mut add) = (k, size);
        if size < k { times = size; add = k; }
        let mut part = 0;
        let mut whole = 0;
        for _ in 0..times {
            part += add;
            while part >= 256 { part -= 256; whole += 1; }
        }
        if b < a { whole = -whole; }
        out(a + whole);
    }
}
