//! Face, once per screen row per frame of the dive. Deck: how many pixels of the row hit the seabed, then for
//! each: the seabed a slice before the hit and at it (tenths over their rows' bases), the seabed a slice before at
//! the hit's x, how far the base drops between the rows (whole units) and how many tenths of the way the ray
//! crossed (0..9). Punches, per pixel: the light on the surface, 20 + 12 slope / 10 32nds of the brightest (8..32,
//! about 0.85 + 0.05 slope over 1.35), the slope (tenths per unit, falling ahead, at the hit's x) within 10 either
//! way; and how rocky it is in 32nds, from the nearer row's height over its base: none at -2 units or below, all at
//! 4.4 or above.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let before = inp();
        let after = inp();
        let above = inp();
        let drop = inp();
        let tenths = inp();
        let mut slope = above - after + drop * 10;
        if slope > 10 { slope = 10; }
        if slope < -10 { slope = -10; }
        out(20 + slope * 12 / 10);
        let mut height = after;
        if tenths < 5 { height = before; }
        height += 20;
        if height < 0 { height = 0; }
        if height > 64 { height = 64; }
        out(height * 5 / 10);
    }
}
