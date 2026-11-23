//! Offset, per row of a jellyfish's halo per frame of the dive. Deck: a place c on the screen from the middle
//! (tenths of a pixel, across, or up), whether that is up (1, else 0), how many, then pixels' columns (or rows,
//! down) counted from the middle's pixel. Punches, per pixel, how far its middle is from c in tenths: 10 x + 5 - c
//! across, 10 x + 5 + c down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut centre = inp();
    if inp() == 1 { centre = -centre; }
    let count = inp();
    for _ in 0..count {
        out(inp() * 10 + 5 - centre);
    }
}
