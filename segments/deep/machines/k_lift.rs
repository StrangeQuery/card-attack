//! Lift, for the dive's ending at 1536x768: the water's light beyond the dial. Deck: how far it has come up (k,
//! 32nds), how many; then for each 4 x 4 pixels and channel its colour, the light there (the table's, by its row of 4)
//! and the dive's depth there (999 open). Punches the colour with fl(light k / 32) added where the water is open.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let k = inp();
    for _ in 0..inp() {
        let mut c = inp();
        let light = inp();
        let depth = inp();
        if depth >= 999 {
            let mut acc = 0;
            for _ in 0..k {
                acc += light;
                while acc >= 32 { acc -= 32; c += 1; }
            }
        }
        out(c);
    }
}
