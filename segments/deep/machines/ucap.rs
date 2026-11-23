//! Cap, once per leaf per frame of the dive (kelp, seagrass). Deck: the leaf's length L (tenths), how many pixels,
//! then for each how far along the leaf it is (tenths). Punches, per pixel, how far past the leaf's ends it is
//! (0 between them) and how far along it is kept within 0..L.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let length = inp();
    let count = inp();
    for _ in 0..count {
        let along = inp();
        if along < 0 { out(-along); out(0); }
        else if along > length { out(along - length); out(length); }
        else { out(0); out(along); }
    }
}
