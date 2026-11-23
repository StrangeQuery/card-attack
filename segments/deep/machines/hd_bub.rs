//! Bubbles, for the dive's creatures at 1536x768: how much of a bubble's light to mix into each pixel it covers.
//! Deck: how much the bubble shows (32nds, at most 24), how far ahead it is (tenths, rounded up), how many pixels,
//! then for each the dive's depth there (whole units, 999 for open water) and the sprite's texel (0..255). Punches,
//! per pixel, texel shows / 255 (32nds, at most 19), none where the seabed is nearer.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let shows = inp();
    let near = inp();
    for _ in 0..inp() {
        let mut depth = inp();
        if depth > 99 { depth = 99; }
        let texel = inp();
        let mut amount = 0;
        if depth * 10 >= near {
            // texel shows can pass a card: the texel added shows times, the 255s carried out as they fill
            let mut sum = 0;
            for _ in 0..shows {
                sum += texel;
                if sum >= 255 { sum -= 255; amount += 1; }
            }
            if amount > 19 { amount = 19; }
        }
        out(amount);
    }
}
