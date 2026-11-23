//! Rays, once per screen row per component (x, y, z) per frame of the dive. Deck: the component of the camera's
//! forward F, right R and up U vectors (each 300 times the unit vector), then the row's B = 95 - 2 row (half
//! pixels up from the middle). The ray through pixel (A, B), A = 2 column - 191, is 300 (f + a r + b u) with
//! a = A / 200 and b = B / 200: (200 F + A R + B U) / 200. Punches that component, rounded down, for the row's
//! 192 pixels.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let forward = inp();
    let right = inp();
    let up = inp();
    let row = inp();
    // the ray as a whole part and two-hundredths: from F, add U B times and -R 191 times to reach the row's left
    // end, then 2 R for each pixel on (A goes up by 2), punching it before each step
    let mut times = row;
    let mut up_step = up;
    if times < 0 { times = -times; up_step = -up; }
    let start = times + 191;
    let mut whole = forward;
    let mut part = 0;
    for i in 0..start + 192 {
        let mut step = up_step;
        if i >= times { step = -right; }
        if i >= start {
            out(whole);
            step = right + right;
        }
        part += step;
        while part >= 200 { part -= 200; whole += 1; }
        while part < 0 { part += 200; whole -= 1; }
    }
}
