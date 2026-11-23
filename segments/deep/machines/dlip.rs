//! Lip, once per screen row per frame of the dive. Deck: the light (32nds), how many pixels of the row hit the
//! seabed, then for each how far the map row before the hit is past the nearest lip (whole units; 99 far from any)
//! and how many tenths of a unit past that row the ray crossed. Each lip catches a faint band of glow, so the next
//! drop-off reads in the dark: punches, per pixel, how much of the band's colour to mix in, in 32nds, falling off
//! over 1.5 units either side of the lip and showing more in the dark: (15 - tenths away) (32 - light) / 27.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let dark = 32 - inp();
    let count = inp();
    for _ in 0..count {
        let away = abs(inp() * 10 + inp());
        let mut amount = 0;
        if away < 15 {
            let mut sum = 0;
            for _ in 0..dark {
                sum += 15 - away;
                while sum >= 27 { sum -= 27; amount += 1; }
            }
        }
        out(amount);
    }
}
