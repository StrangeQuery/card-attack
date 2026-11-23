//! The tumble's levels, once a frame. Deck: the frame f. Punches the faces' fizzle level (0..16, one a frame
//! once it travels), the rotozoomer's dissolve level and frame (from 120 frames into the travel), the cube's
//! dissolve level (out, from 154): each dissolve's level k is 16 (k + 1) / 17 through its 16 frames.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

const FLAT: i16 = 16;
const TILT: i16 = 15;
const ROTO_IN: i16 = 120;
const CUBE_OUT: i16 = 154;

/// Level k of a 16-frame dissolve: 0 before it, 16 (k + 1) / 17 through it, then 16.
fn level(k: i16) -> i16 {
    if k < 0 { return 0; }
    if k >= 16 { return 16; }
    // 16 (k + 1), as doublings
    let mut x = k + 1;
    x += x;
    x += x;
    x += x;
    x += x;
    let mut q = 0;
    while x >= 17 { x -= 17; q += 1; }
    q
}

pub fn main() {
    let f = inp();
    let v = f - FLAT - TILT;
    if v < 0 { out(0); out(0); out(0); out(0); return; }
    out(if v + 1 < 16 { v + 1 } else { 16 });
    out(level(v - ROTO_IN));
    out(if v > ROTO_IN { v - ROTO_IN } else { 0 });
    out(level(v - CUBE_OUT));
}
