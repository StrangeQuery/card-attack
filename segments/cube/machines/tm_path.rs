//! The tumble's path, once a frame. Deck: the frame f. Punches whether the cube travels a path (1 or 0), the
//! path's three steps (120ths: across round once in 60 frames and toward us and away a quarter behind, up and down
//! once in 30) and the lens (from the path's far end, 504, closing in 20 a frame to 205).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

const FLAT: i16 = 16;
const TILT: i16 = 15;
const LENS: i16 = 205;
const FAR: i16 = 504;

/// x mod m, by taking m away (x 0 or more).
fn modulo(mut x: i16, m: i16) -> i16 {
    while x >= m { x -= m; }
    x
}

pub fn main() {
    let f = inp();
    if f < FLAT { out(0); out(0); out(0); out(0); out(0); return; }
    let k = f - FLAT;
    if k < TILT { out(1); out(0); out(0); out(30); out(FAR); return; }
    let v = k - TILT;
    out(1);
    out(modulo(2 * v, 120));
    out(modulo(4 * v, 120));
    out(modulo(30 + 2 * v, 120));
    // max(LENS, FAR - 20 v): from v 15 on, 20 v is past the gap
    out(if v > 14 { LENS } else { FAR - 20 * v });
}
