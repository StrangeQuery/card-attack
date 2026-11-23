//! Mix, per screen row per channel per frame of the dive, once for each way colours are mixed (rock into sand, the
//! light on the surface, the caustics, the fog, the glows). Deck: how many pixels, then for each a, b (0..255) and
//! k (0..32). Punches, per pixel, a + (b - a) k / 32, toward zero.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let k = inp();
        // from a toward b by k 32nds of the difference, rounded down; or, when 32 - k is fewer, from b back toward
        // a by 32 - k 32nds, rounded up (31 32nds to start with)
        let mut from = a;
        let mut to = b;
        let mut times = k;
        let mut part = 0;
        if k > 16 { from = b; to = a; times = 32 - k; part = 31; }
        let mut whole = 0;
        if times > 0 {
            // (b - a) k can pass a card. The size of the difference is 32 q + r, so n times it is n q and n r / 32:
            // n q as the 32s are taken off, then r added n times (counting n down to zero), the 32s carried out as
            // they fill.
            let mut rest = abs(to - from);
            while rest >= 32 { rest -= 32; whole += times; }
            while times > 0 {
                times -= 1;
                part += rest;
                if part >= 32 { part -= 32; whole += 1; }
            }
            if to < from { whole = -whole; }
        }
        out(from + whole);
    }
}
