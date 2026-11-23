//! Gauss, once per row of a jellyfish's halo per frame of the dive. Deck: the row's distance up from the middle
//! over the halo's width (hundredths), how many pixels, then for each its distance across over the width
//! (hundredths). Punches, per pixel, q = (u^2 + v^2) / 20, u and v those distances in tenths: the halo's fall-off
//! is looked up by it, and none from 49 on (49 for any further; the squares are only added while they stay
//! under 980, so that nothing passes a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let up = abs(inp() / 10);
    let count = inp();
    let mut room = 980;                 // what the across part's square may be
    if up > 31 { room = -1; } else { for _ in 0..up { room -= up; } }
    for _ in 0..count {
        let across = abs(inp() / 10);
        let mut q = 49;
        if across <= 31 && room >= 0 {
            let mut square = 0;
            for _ in 0..across { square += across; }
            if square < room { q = (980 - room + square) / 20; }
        }
        out(q);
    }
}
