//! Whether our car covers a pixel of the accelerated half, drawn at twice the size: if it covers at least 3 of the
//! pixel's 4 samples, the pixel takes the car's distance. Deck: how many pixels, then each its 4 samples, 1 where the
//! car is, else 0. Punches each 1 if at least 3 are the car's, else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let pixels = inp();
    for _ in 0..pixels as usize {
        let covered = inp() + inp() + inp() + inp();
        out(if covered >= 3 { 1 } else { 0 });
    }
}
