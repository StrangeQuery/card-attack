//! Key, per row of each frame the cube is laid over a picture (the BBS, the rotozoomer): where the cube is. Deck:
//! how many pixels, then each one's red, green and blue (0..255) from the cube's frames, black where nothing is.
//! Punches 1 for each pixel that isn't black, else 0: the cube's there, in front of the picture.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let n = inp();
    for _ in 0..n {
        let mut here = 0;
        for _ in 0..3 { if inp() > 0 { here = 1; } }
        out(here);
    }
}
