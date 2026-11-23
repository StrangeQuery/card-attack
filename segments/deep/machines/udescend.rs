//! Descend, a few times per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed,
//! then for each: the ray's height a slice before the hit and at it (over the hit's row base, whole units and
//! hundredths 0..99) and a level (whole units). Punches, per pixel, how many tenths of the slice the ray takes to
//! go down to the level: 10 a / (a + b), a the tenths it was over it (at most 99), b the tenths under it after; 0
//! when it was not over it a slice before, 10 when it never goes under.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let before_whole = inp();
        let before = before_whole * 10 + inp() / 10;
        let after_whole = inp();
        let after = after_whole * 10 + inp() / 10;
        let top = inp() * 10;
        let mut tenths = 10;
        if after < top {
            tenths = 0;
            let mut above = before - top;
            if above > 99 { above = 99; }
            if above > 0 {
                let whole = above + top - after;
                let mut left = above * 10;
                while left >= whole { left -= whole; tenths += 1; }
            }
        }
        out(tenths);
    }
}
