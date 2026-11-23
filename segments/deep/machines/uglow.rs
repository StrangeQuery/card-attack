//! Glow, once per screen row per frame of the dive. Deck: how dark it is, 32 - 7 light / 10 (light in 32nds), how
//! many pixels of the row hit the seabed on a glowing spot, then for each how bright the spot is there (32nds) and
//! its pulse, 20 + 20 sin (0..40). The glow pulses between 0.2 and 1 and shows more in the dark. Punches, per pixel,
//! how much of the spot's colour to mix in, in 32nds: 3 / 4 of the glow (the pulsed brightness, 3 dark / 128),
//! at most 19.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let dark = inp();
    let count = inp();
    for _ in 0..count {
        let bright = inp();
        let pulse = inp() + 10;         // 0.2 + 0.8 of the pulse, in 50ths
        let mut amount = by(by(bright, pulse, 50), dark * 3, 128);   // and 3 / 4 by the dark
        if amount > 19 { amount = 19; }
        out(amount);
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
