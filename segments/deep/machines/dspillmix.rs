//! Spill mix, per screen row per channel per frame of the dive while the portal shows: its spill of light mixed in,
//! finer than 32nds (it fades out too gently for them). Deck: how far the portal has come on (32nds), how many
//! pixels, then for each a, b (0..255) and the spill there in tenths of 32nds (0..110, from the spill machine). The
//! amount is on spill / 40, in 256ths (the 40s carried out as they fill). Punches, per pixel, a + (b - a) amount /
//! 256, toward zero: the difference added that many times, the 256s carried out as they fill.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let on = inp();
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let spill = inp();
        let mut amount = 0;
        let mut sum = 0;
        for _ in 0..on {
            sum += spill;
            while sum >= 40 { sum -= 40; amount += 1; }
        }
        let size = abs(b - a);
        let mut part = 0;
        let mut whole = 0;
        for _ in 0..amount {
            part += size;
            if part >= 256 { part -= 256; whole += 1; }
        }
        if b < a { whole = -whole; }
        out(a + whole);
    }
}
