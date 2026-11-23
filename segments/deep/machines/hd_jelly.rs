//! Jellyfish, for the dive's creatures at 1536x768: each pixel of a jellyfish's glow made a cover and a colour. Deck:
//! its fade (64ths), how far ahead it is (tenths), how many pixels, then for each the dive's depth there (whole units,
//! 999 for open water), the cover for its glow (64ths, from the table) and how far toward the pale edge colour its
//! glow and its rim take it (32nds, from the tables). Punches, per pixel, how much to mix in (cover fade / 128,
//! 32nds, at most 31; none where the seabed is more than 2 units nearer) and how pale (32nds, at most 32).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let fade = inp();
    let ahead = inp();
    for _ in 0..inp() {
        let mut depth = inp();
        if depth > 99 { depth = 99; }
        let cover = inp();
        let pale = inp() + inp();
        let mut amount = 0;
        if depth * 10 >= ahead - 20 {
            // cover fade can pass a card: added fade times, the 128s carried out as they fill
            let mut sum = 0;
            for _ in 0..fade {
                sum += cover;
                if sum >= 128 { sum -= 128; amount += 1; }
            }
            if amount > 31 { amount = 31; }
        }
        out(amount);
        if pale > 32 { out(32); } else { out(pale); }
    }
}
