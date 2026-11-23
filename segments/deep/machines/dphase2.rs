//! Phase across, once per screen row per frame of the dive. Deck: the third caustic wave's phase this frame
//! (0..59), how many pixels of the row hit the seabed, then for each: x where the ray crossed it (a whole part,
//! -150..300, and hundredths), how many tenths of a slice it crossed after the row before, and that row's z
//! (0..450). The wave runs across x and z together, 7.5 units long, on a 60-step circle. Punches, per pixel, its
//! step (0..59).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let phase = inp();
    let count = inp();
    for _ in 0..count {
        let x = inp();
        let x_tenths = inp() / 10;
        let tenths = inp();
        let z = inp();
        // SAFETY: the host deals x from -150 up and z from 0 up
        unsafe { assume(x >= -150 && x <= 300 && z >= 0 && z <= 450) };
        let mut across = x + z + 150;
        while across >= 15 { across -= 15; }
        let mut tenths_across = 10 * across + x_tenths + tenths;
        while tenths_across >= 75 { tenths_across -= 75; }
        let mut step = tenths_across * 8 / 10 + phase;
        if step >= 60 { step -= 60; }
        out(step);
    }
}
