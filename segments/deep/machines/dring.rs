//! Ring, once per screen row per frame of the dive while the portal shows. Deck: how far the portal has come on
//! (32nds), how many pixels, then for each how far it is from the middle (tenths of a unit) and how strongly its
//! swirl shows (32nds). Inside the ring the swirl shows by the portal's on; the ring is a bright band 0.45 either
//! side of 6 units out. Punches, per pixel, how much of the swirl and of the ring's colour to mix in (32nds).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let on = inp();
    let count = inp();
    for _ in 0..count {
        let far = inp() - 60;
        let strength = inp();
        let mut swirl = 0;
        if far < 0 { swirl = by(on, strength, 32); }
        out(swirl);
        let edge = abs(far);
        let mut ring = 0;
        if edge < 5 { ring = by(on, 9 - edge - edge, 9); }
        out(ring);
    }
}

/// a b / d rounded down, for a b small: a added b times, the d's counted as they fill.
#[inline(never)]
fn by(a: i16, b: i16, d: i16) -> i16 {
    let mut part = 0;
    let mut whole = 0;
    for _ in 0..b {
        part += a;
        while part >= d { part -= d; whole += 1; }
    }
    whole
}
