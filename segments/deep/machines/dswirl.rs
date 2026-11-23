//! Swirl, once per screen row per frame of the dive while the portal shows. Deck: the swirl's and the colour's
//! phases this frame (tenths of a step of a 60-step turn, 0..599), how many pixels, then for each which way and how
//! far it is from the portal's middle (tenths of a step; tenths of a unit). The swirl turns three times round the
//! middle and winds outward, 12.4 tenths of a step per tenth of a unit (1.3 radians a unit); the colour turns twice
//! round. Punches, per
//! pixel, the swirl's step and the colour's step (0..59).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let swirl = inp();
    let colour = inp();
    let count = inp();
    for _ in 0..count {
        let angle = inp();
        let mut far = inp();
        if far > 60 { far = 60; }
        let twice = round(angle, angle);
        let mut wind = far * 12 + far * 4 / 10;
        if wind >= 600 { wind -= 600; }
        out(round(round(twice, angle), round(wind, swirl)) / 10);
        out(round(twice, colour) / 10);
    }
}

/// a + b round a turn of 600, for a and b 0..599 (taking 600 - b off first, so it never passes a card).
#[inline(never)]
fn round(a: i16, b: i16) -> i16 {
    let sum = a - (600 - b);
    if sum < 0 { sum + 600 } else { sum }
}
