//! Fade, for an item or a group of a UI layer laid on faded: each pixel's cover taken down. Deck: the fade (0-32),
//! how many pixels, then each one's cover (0-32). Punches each cover times fade / 32, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let fade = inp();
    let count = inp();
    let mut was = -1;
    let mut faded = 0;
    for _ in 0..count {
        let cover = inp();
        // the same cover as the last pixel's is faded the same
        if cover != was {
            was = cover;
            faded = muldiv(cover, fade, 32);
        }
        out(faded);
    }
}
