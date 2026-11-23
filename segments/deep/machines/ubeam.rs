//! Beam, once per screen row per frame of the dive. Deck: the screen row (0..95), where the diver's torch points (its
//! beam's middle from the screen's middle, half pixels right and up), how many pixels of the row hit the seabed, then
//! each one's column (0..191). Punches, per pixel, how much of the beam falls there in 32nds: the distance from the
//! beam's middle in half pixels taken as the larger offset and half the smaller, r, and the beam 40 - r, within
//! 0..32: full within 4 pixels, fading out by 20 (brighter, much narrower).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let row = inp();
    let aim_across = inp();
    let aim_up = inp();
    let mut up = row * 2 - 95 + aim_up;
    if up < 0 { up = -up; }
    let count = inp();
    for _ in 0..count {
        let mut across = inp() * 2 - 191 - aim_across;
        if across < 0 { across = -across; }
        let mut big = across;
        let mut small = up;
        if up > across { big = up; small = across; }
        let mut beam = 40 - big - small / 2;
        if beam < 0 { beam = 0; }
        if beam > 32 { beam = 32; }
        out(beam);
    }
}
