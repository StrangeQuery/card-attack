//! Fish cover, for the dive's creatures at 1536x768: how much of a fish to mix into each pixel it covers. Deck: how
//! far ahead the fish is (tenths, rounded up), how many pixels, then for each the dive's depth there (whole units, 999
//! for open water) and the texel's cover (0..255). Punches, per pixel, the cover's 32nds, none where the seabed is
//! nearer.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let near = inp();
    for _ in 0..inp() {
        let mut depth = inp();
        if depth > 99 { depth = 99; }
        let cover = inp();
        // halved three times: a half is a short sum on CARDIAC, an eighth a long division
        if depth * 10 >= near { out(cover / 2 / 2 / 2); } else { out(0); }
    }
}
