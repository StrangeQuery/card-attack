//! XOR, once for each of the Finder's zoom rects in a frame: its outline turns over every pixel it crosses, black to
//! white and white to black. Deck: how many pixels, then each (1 black, 0 white). Punches each turned over.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        out(1 - inp());
    }
}
