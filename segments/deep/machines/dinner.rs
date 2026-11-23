//! Inner, once per screen row per frame of the dive while the portal shows. Deck: how many pixels, then for each how
//! far it is from the portal's middle (tenths of a unit) and its swirl's and colour's waves, 20 + 20 sin (0..40).
//! The swirl is brighter toward the rim: (wave) (0.6 + 0.4 r / 6). Punches, per pixel, how much of the swirl's light
//! to mix into its colour (32nds), how much violet to mix into the cyan (32nds), and how strongly the swirl shows
//! inside the ring: 0.55 + 0.35 swirl, as 18 + 11 swirl / 32 (32nds).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut reach = inp();
        let wave = inp();
        let hue = inp();
        // wave (90 + r) / 188, by adding, the 188s counted as they fill
        if reach > 60 { reach = 60; }
        let mut sum = 0;
        let mut swirl = 0;
        for _ in 0..90 + reach {
            sum += wave;
            if sum >= 188 { sum -= 188; swirl += 1; }
        }
        out(swirl);
        out(hue * 8 / 10);
        out(18 + swirl * 11 / 32);
    }
}
