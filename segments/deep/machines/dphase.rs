//! Phase, once per screen row per frame of the dive. Deck: two of the caustics' phases this frame (0..59), how
//! many pixels of the row hit the seabed, then for each: x where the ray crossed it (a whole part, -150..300, and
//! hundredths), how many tenths of a slice it crossed after the row before, and that row's z (0..450). Two of the
//! waves run across x (5 units long) and along z (6) on a 60-step circle. Punches, per pixel, their steps
//! (0..59).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let phase_x = inp();
    let phase_z = inp();
    let count = inp();
    for _ in 0..count {
        let mut x = inp() + 200;
        let x_tenths = inp() / 10;
        let tenths = inp();
        let mut z = inp();
        // SAFETY: the host deals x from -150 up and z from 0 up
        unsafe { assume(x >= 50 && x <= 500 && z >= 0 && z <= 450) };
        while x >= 5 { x -= 5; }
        while z >= 6 { z -= 6; }
        out(wrap60((10 * x + x_tenths) * 12 / 10 + phase_x));
        out(wrap60(10 * z + tenths + phase_z));
    }
}

/// i brought into 0..59, for i in 0..119.
fn wrap60(i: i16) -> i16 { if i >= 60 { i - 60 } else { i } }
