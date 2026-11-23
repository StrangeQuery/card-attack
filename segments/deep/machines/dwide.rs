//! Wide, once per leaf per frame of the dive (kelp, seagrass). Deck: how many pixels, then for each how far it is
//! from the leaf (tenths) and the leaf's half width there (tenths). Punches, per pixel, 1 if it is within the width
//! (at least 5 tenths), else 0, and that width.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let distance = inp();
        let mut width = inp();
        if width < 5 { width = 5; }
        out(if distance <= width { 1 } else { 0 });
        out(width);
    }
}
