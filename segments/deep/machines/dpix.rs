//! Pixel, per frame of the dive for the jellyfish's tentacles. Deck: across (1) or up (-1), how many, then places
//! on the screen from the middle (tenths of a pixel). Punches, per place, its pixel counted from the middle's:
//! columns place / 10 across, rows -place / 10 down, rounded down (the screen's middle pixel is column 96, row 48).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let across = inp();
    let count = inp();
    for _ in 0..count {
        let mut v = inp();
        if across != 1 { v = -v; }
        let q = v / 10;                 // rounded down
        if q * 10 > v { out(q - 1); } else { out(q); }
    }
}
