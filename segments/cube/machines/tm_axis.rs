//! The tumble's axis and tilt, once a frame. Deck: the frame f. Punches the axis (2 while flat, 0 while it
//! tilts, then about x for 20 frames of the travel, y for the next 20, and so on) and the tilt (0, then a step every
//! 4 frames to 3).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

const FLAT: i16 = 16;
const TILT: i16 = 15;

pub fn main() {
    let f = inp();
    let k = f - FLAT;
    if f < FLAT { out(2); out(0); return; }
    let mut q = 0;
    if k < TILT {
        let mut x = k;
        while x >= 4 { x -= 4; q += 1; }
        out(0);
        out(if q < 3 { q } else { 3 });
        return;
    }
    let mut x = k - TILT;
    let mut odd = 0;
    while x >= 20 { x -= 20; odd = 1 - odd; }
    out(1 - odd);
    out(3);
}
