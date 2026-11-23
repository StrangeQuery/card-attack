//! Shade, per frame of the dive for the glows drawn over the seabed (the jellyfish's halos, bells and tentacles).
//! Deck: the most to mix in (32nds), how many, then for each a strength (at most 999 / its fade), a fade f (at most 64), how far it is ahead
//! (tenths) and how far the seabed is in its pixel (whole units; 999 for open water). Punches, per glow, strength f
//! / 64 (32nds to mix in), at most the most; 0 where the seabed is more than 2 units nearer.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let most = inp();
    let count = inp();
    for _ in 0..count {
        let strength = inp();
        let fade = inp();
        let ahead = inp();
        let mut seabed = inp();
        if seabed > 99 { seabed = 99; }
        let mut amount = 0;
        if seabed * 10 >= ahead - 20 {
            let mut sum = 0;
            for _ in 0..fade {
                sum += strength;
                while sum >= 64 { sum -= 64; amount += 1; }
            }
        }
        if amount > most { amount = most; }
        out(amount);
    }
}
