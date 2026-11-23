//! Water, once per screen row per frame of the dive. Deck: how many pixels of the row see open water, then for each
//! how far its ray points up, in hundredths (-100..100). The water is bright above and dark below, fixed to the
//! world: punches, per pixel, how much of the bright colour to mix into the dark, in 32nds: 16 + 3 up / 10, from 0
//! to 32.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut level = 16 + inp() * 3 / 10;
        if level < 0 { level = 0; }
        if level > 32 { level = 32; }
        out(level);
    }
}
