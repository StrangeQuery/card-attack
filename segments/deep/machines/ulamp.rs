//! Lamp, once per screen row per frame of the dive. Deck: the light (32nds), how many pixels of the row hit the
//! seabed, then for each how much of the lamp's beam falls there (32nds) and how far it is (the fog's distance).
//! Punches, per pixel, how far toward its colour in full light the diver's lamp takes the rock, in 32nds: 70 -
//! distance (at most 26, none past 70), times the beam / 32, times (32 - light) / 32, each rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let dark = 32 - inp();
    let count = inp();
    for _ in 0..count {
        let beam = inp();
        let mut near = 70 - inp();
        if near < 0 { near = 0; }
        if near > 26 { near = 26; }
        let mut sum = 0;
        let mut lit = 0;
        for _ in 0..beam {
            sum += near;
            if sum >= 32 { sum -= 32; lit += 1; }
        }
        sum = 0;
        let mut amount = 0;
        for _ in 0..dark {
            sum += lit;
            if sum >= 32 { sum -= 32; amount += 1; }
        }
        out(amount);
    }
}
