//! Cover, once for each item of a UI layer (a box's rounded edges, a shadow, a line of text, a sprite), and for a
//! group laid on faded: how each of its pixels covers what is under it in the layer. Deck: how many pixels; then
//! each: its cover (0-32) and the layer's cover under it. Punches for each: the cover the two make together, and the
//! pixel's share of that in 32nds (how far the mix machine takes the colour from the layer's toward the pixel's).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let cover = inp();
        let under = inp();
        let mut both = under;
        let mut share = 0;
        if cover == 32 || (under == 0 && cover > 0) {
            both = cover;
            share = 32;
        } else if cover > 0 {
            // under + (32 - under) cover / 32, then 32 cover / both: without the products (muldiv)
            both += muldiv(cover, 32 - under, 32);
            share = muldiv(cover, 32, both);
        }
        out(both);
        out(share);
    }
}
