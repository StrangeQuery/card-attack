//! Halo, for the dive's jellyfish at 1536x768: how much of the glow to mix into each 4 x 4 pixels round a
//! jellyfish. Deck: its fade (64ths), how far ahead it is (tenths), how many, then for each the dive's depth there
//! (whole units, 999 for open water) and the fall-off from the table (0..56). Punches, per one, fall-off fade / 40
//! (256ths), none where the seabed is more than 2 units nearer.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let fade = inp();
    let ahead = inp();
    for _ in 0..inp() {
        let mut depth = inp();
        if depth > 99 { depth = 99; }
        let fall = inp();
        let mut amount = 0;
        if depth * 10 >= ahead - 20 {
            // fall fade can pass a card: added fade times, the 40s carried out as they fill
            let mut sum = 0;
            for _ in 0..fade {
                sum += fall;
                while sum >= 40 { sum -= 40; amount += 1; }
            }
        }
        out(amount);
    }
}
