//! Far, once per screen row per frame of the dive. Deck: the first slice's place ahead of the camera s (tenths,
//! 1..10), how many pixels of the row hit the seabed, then for each: how many tenths of the way the ray crossed
//! (0..9) after the slice before the hit, the slice k of the hit and the ray's forward part d (100..441). Punches,
//! per pixel, how far the hit is along the view, 300 (k - 1 + (s + tenths) / 10) / d at least 0, and the fog
//! there in 32nds, as 0.85 (d / 70)^0.8 up to all of it, in two straight pieces: d / 2 up to 20, then
//! 10 + (d - 20) / 3.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let start = inp();
    let count = inp();
    for _ in 0..count {
        let tenths = inp();
        let slice = inp();
        let ahead = inp();
        // 30 (10 (k - 1) + s + tenths) / d without the product: 300 per slice added, d taken away as it fits
        let mut sum = (start + tenths) * 30 - 300;
        let mut along = 0;
        for _ in 0..slice + 1 {
            while sum >= ahead { sum -= ahead; along += 1; }
            sum += 300;
        }
        out(along);
        let mut fog = along / 2;
        if along >= 20 { fog = 10 + (along - 20) / 3; }
        if fog > 32 { fog = 32; }
        out(fog);
    }
}
