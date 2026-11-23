//! Coral, once per screen row per frame of the dive. Deck: how many pixels of the row hit the seabed, then for each:
//! its place in the rock's tables (0..15), its tone (0..3), the grain there (32nds lighter; 0 far), the light on it
//! and how much lighter its column is (32nds), and whether it is rubble or a ledge's lip (1, 2; else 0). Rubble is 5 lighter or darker again with the
//! grain; one lip in four (by place) has coral where the grain is light (tone 4). Punches, per pixel: the tone and the
//! light with the grain (0..32).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut quarter = inp();
        let mut tone = inp();
        let grain = inp();
        let light_face = inp();
        let mut light = inp() + light_face;
        light += grain;
        let detail = inp();
        if detail == 1 {
            if grain >= 0 { light += 5; } else { light -= 5; }
        }
        while quarter >= 4 { quarter -= 4; }
        if detail == 2 && quarter == 0 && grain >= 0 { tone = 4; }
        if light < 0 { light = 0; }
        if light > 32 { light = 32; }
        out(tone);
        out(light);
    }
}
