//! Lay, once per row per frame of the polys: a picture laid over the scene where it shows (the brag box, the words).
//! Deck: the count, then for each pixel 1 if the picture shows there (else 0), the picture's red, green and blue,
//! then the scene's. Punches the picture's where it shows, else the scene's.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let shows = inp();
        let top = read_pixel();
        let under = read_pixel();
        punch(if shows > 0 { top } else { under });
    }
}
