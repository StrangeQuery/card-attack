//! Dissolve, once per screen row of a title card's pixels per frame: HyperCard's dissolve, each pixel showing when its
//! turn comes. Deck: how far the dissolve has come (85 a third: 85, 170 or 255), how many of the card's pixels are in
//! the row, then each one's turn (0-254, the dissolve's order). Punches 1 for each that shows, 0 for each still to.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let come = inp();
    let count = inp();
    for _ in 0..count {
        if inp() < come { out(1); } else { out(0); }
    }
}
