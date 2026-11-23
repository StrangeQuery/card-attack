//! Leaf, per frame of the dive for the kelp's and seagrass's leaves. Deck: how many leaves, then for each its two
//! ends on the screen (tenths of a pixel from the middle, across and up). Punches, per leaf, its start and end
//! across and down, and how far it runs across and down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let ax = inp();
        let ay = inp();
        let bx = inp();
        let by = inp();
        out(ax);
        out(-ay);
        out(bx);
        out(-by);
        out(bx - ax);
        out(ay - by);
    }
}
