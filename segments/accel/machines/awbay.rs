//! Wall bays, as many as the host has, per frame of the accelerated drive: which column of the window texture a texel
//! column of a wall is in, and where in its bay (8 texels across). Deck: how many, then each: the texel across, the
//! wall's window texture place, 1 for an end wall. Punches each the texture's column (the place, 480 on for an end
//! wall, 32 a bay, round 960) and the texel's place in its bay (0-7).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut across = inp();
        // 480 on round 960 (a sum past 999 would not hold)
        let mut place = inp();
        if inp() > 0 { if place >= 480 { place -= 480; } else { place += 480; } }
        while across >= 8 {
            across -= 8;
            place += 32;
            if place >= 960 { place -= 960; }
        }
        out(place);
        out(across);
    }
}
