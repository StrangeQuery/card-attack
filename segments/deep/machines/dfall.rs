//! Falloff, once per screen row per frame of the dive. Deck: the spots' pulse this frame (a step 0..59 of a 60-step
//! turn), how many pixels of the row hit the seabed on a glowing spot, then for each where the ray crossed it (the
//! hundredths of x and the tenths of a unit past the map row before) and the spot's pulse (0..9). A spot sits in the
//! middle of its unit square; its glow falls off with the squared distance from there. Punches, per pixel, how
//! bright the spot is there (32nds) and how far round its pulse is (a step 0..59).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let falloff = tables::GLOW_FALLOFF;
    let pulse = inp();
    let count = inp();
    for _ in 0..count {
        let across = abs(inp() / 10 - 5);   // tenths from the middle of the square, 0..5
        let along = abs(inp() - 5);
        let mut reach = 0;                  // the squared distance, by adding
        for _ in 0..across { reach += across; }
        for _ in 0..along { reach += along; }
        // SAFETY: at most 50, a fifth of it at most 10
        unsafe { assume(reach >= 0 && reach <= 50) };
        out(falloff[(reach / 5) as usize]);
        let mut step = pulse + inp() * 6;
        if step >= 60 { step -= 60; }
        out(step);
    }
}
