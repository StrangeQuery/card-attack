//! Software colours' choice, once per channel per frame of the software half: the level shown, through the ordered
//! dither. Deck: how many, then each: the colour (tenths), the threshold from which the next level up shows (data
//! softq.txt, at its level and the block's Bayer value), its level. Punches each the level, or the next up.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let t = inp();
        let th = inp();
        let i = inp();
        out(if t >= th { i + 1 } else { i });
    }
}
