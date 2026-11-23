//! Street places, once per row of the street per frame of the accelerated drive: for each pixel, its place in the
//! street's cross-section (data/cross.txt: what lies how far from the middle, and which pixels are near enough an
//! edge to be partly covered). Deck: how many pixels, then for each (asrow's) its distance (quarters; -1: no street) and
//! how far from the middle (64ths, signed). Punches for each: -1 for no street, else how near an edge counts (a / 32
//! + 3, 64ths: at least half the pixel's width) and how far from the middle (64ths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let far = inp();
        let off = abs(inp());
        if far < 0 { out(-1); } else {
            out(far / 2 / 2 / 2 / 2 / 2 + 3);
            out(off);
        }
    }
}
