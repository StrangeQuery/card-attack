//! Glow, for the dive's creatures at 1536x768: a strand's piece adding its glow to the pixels it covers. Deck: the
//! piece's strength f (hundredths, at most 299), how many pixels, then for each the glow there so far and the share
//! of the strength there (hundredths, at most 100). Punches, per pixel, the glow plus f share / 100 (by f's digits,
//! ((c s / 10 + b s) / 10 + a s)), at most 299.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let f = inp();
    let (a, b, c) = (digit(f, 0), digit(f, 1), digit(f, 2));
    for _ in 0..inp() {
        let glow = inp();
        let share = inp();
        // each digit's product by adding the share that many times (at most 9)
        let (mut ca, mut ba, mut aa) = (0, 0, 0);
        for _ in 0..c { ca += share; }
        for _ in 0..b { ba += share; }
        for _ in 0..a { aa += share; }
        let mut sum = glow + (ca / 10 + ba) / 10 + aa;
        if sum > 299 { sum = 299; }
        out(sum);
    }
}
