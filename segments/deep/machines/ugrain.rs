//! Grain, once per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed, then for
//! each: its place in the rock's tables (0..15), the hit's x within its unit (hundredths), how many tenths of the
//! slice in, the face (0 a top, else a front or side) and the hit's height's hundredths. Close up each face has a
//! grain of 4 x 4 chips and cracks: a table of 16, from the place, across the unit and down a face or along a top.
//! Punches, per pixel, how much lighter the grain is there (32nds).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let place = inp();
        let across = inp() / 25;
        let tenths = inp();
        let face = inp();
        let mut down = inp() / 25;
        if face == 0 { down = tenths * 4 / 10; }
        let mut at = place + across + down * 4;
        while at >= 16 { at -= 16; }
        // SAFETY: 0..15
        unsafe { assume(at >= 0 && at < 16) };
        out(tables::U_ROCK_GRAIN[at as usize]);
    }
}
