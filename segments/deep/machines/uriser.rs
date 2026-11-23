//! Riser, once per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed, then for
//! each: the face (0 a top, 1 a front, 2 or 3 a side facing -x or +x), the block in front and the block of the column
//! a slice before (whole units over the hit's row base), the hit's height (whole units and hundredths 0..99), and the
//! light so far (32nds). A front's light is 16, a side's 12 or 22, 5 less within 0.35 of the top it rises from (the
//! block in front for a front, the column a slice before for a side). Punches, per pixel, the light (a top's as it
//! was).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let face = inp();
        let mut low = inp();
        let low_side = inp();
        let height = inp();
        let height_part = inp();
        let mut light = inp();
        if face != 0 {
            light = 16;
            if face == 2 { light = 12; low = low_side; }
            if face == 3 { light = 22; low = low_side; }
            let over = height - low;
            if over < 0 { light -= 5; }
            if over == 0 && height_part < 35 { light -= 5; }
        }
        out(light);
    }
}
