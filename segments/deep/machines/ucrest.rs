//! Crest, once per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed, then for
//! each: the face (0 a top, else a front or side), its block's top (whole units over the hit's row base), the hit's
//! height (whole units and hundredths 0..99) and the rim so far (32nds). A face's top edge catches 7 more within 0.15
//! of the top. Punches, per pixel: the rim and the hit's height (on a top, its top), whole units and hundredths.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let face = inp();
        let top = inp();
        let mut height = inp();
        let mut height_part = inp();
        let mut rim = inp();
        if face == 0 {
            height = top;
            height_part = 0;
        } else {
            let under = top - height;
            if under <= 0 { rim += 7; }
            if under == 1 && height_part > 85 { rim += 7; }
        }
        out(rim);
        out(height);
        out(height_part);
    }
}
