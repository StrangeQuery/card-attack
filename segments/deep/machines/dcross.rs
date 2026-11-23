//! Cross, once per screen row per frame of the dive. Deck: how many pixels of the row hit the seabed, then for
//! each the ray's height a slice before the hit (whole units and hundredths over that row's base) and the seabed
//! there (tenths), then the ray's height at the hit (over its row's base) and the seabed there, which it is under.
//! Punches, per pixel, how many tenths of the way from the slice before to the hit the ray crossed the seabed:
//! 10 a / (a + b), a the height over the seabed before (at most 99 tenths; none above it counts as 0) and b the
//! depth under it after.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let before = inp();
        let before_part = inp();
        let seabed_before = inp();
        let after = inp();
        let after_part = inp();
        let seabed_after = inp();
        let mut above = before * 10 + before_part / 10 - seabed_before;
        if above > 99 { above = 99; }
        let under = seabed_after - after * 10 - after_part / 10;
        let mut tenths = 0;
        if above > 0 {
            // 10 above / (above + under), at most 9: count the tenths that fit
            let mut left = above * 10;
            let whole = above + under;
            while left >= whole { left -= whole; tenths += 1; }
        }
        out(tenths);
    }
}
