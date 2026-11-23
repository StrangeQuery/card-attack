//! Ray, once per screen row per frame of the dive. Deck: the light (32nds), how many pixels of the row see open
//! water, then for each its two sharpened god-ray waves (40ths) and how far its ray points up (hundredths). Shafts
//! are seen only looking up, by up + 0.2. Punches, per pixel, how much of the shafts' colour to mix in, in 32nds:
//! about 0.45 (both waves) (up + 0.2), (waves) (up + 20) / 10 / 28, at most 16, by the light.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let light = inp();
    let count = inp();
    for _ in 0..count {
        let shafts = inp() + inp();
        let up = (inp() + 20) / 10;
        // (waves) (up) / 28 and then by the light, both by adding
        let mut sum = 0;
        let mut amount = 0;
        for _ in 0..up {
            sum += shafts;
            while sum >= 28 { sum -= 28; amount += 1; }
        }
        if amount > 16 { amount = 16; }
        sum = 0;
        let mut lit = 0;
        for _ in 0..light {
            sum += amount;
            if sum >= 32 { sum -= 32; lit += 1; }
        }
        out(lit);
    }
}
