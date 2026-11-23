//! Tip, per kelp blade per frame of the dive. Deck: the blade's base on the screen (tenths of a pixel from the
//! middle, across and up) and how far its tip is from it across and down (tenths). Punches 1 and the tip, across
//! and up; or 0, 0, 0 where the tip would be more than 999 tenths from the middle (off the screen; the blade is not
//! drawn), found without passing a card.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let x = inp();
    let y = inp();
    let across = inp();
    let down = inp();
    if fits(x, across) && fits(y, -down) {
        out(1);
        out(x + across);
        out(y - down);
    } else {
        out(0);
        out(0);
        out(0);
    }
}

/// Whether a + b stays within 999 either way (a and b each within 999).
#[inline(never)]
fn fits(a: i16, b: i16) -> bool {
    if a >= 0 { b <= 999 - a } else { b >= -999 - a }
}
