//! Which thing laid over a pixel of the accelerated drive shows: the nearest of them, if it is nearer than what the
//! pixel already shows (of things equally near, the first). The host groups the things by pixel, keeping their
//! order, and copies the winners in. Deck: how many pixels, then each: the pixel's distance, how many things, and
//! each thing's distance (quarters). Punches each pixel's winner, 1 for the first thing, or 0 for none.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let pixels = inp();
    for _ in 0..pixels as usize {
        let mut nearest = inp();
        let things = inp();
        let mut winner = 0;
        for t in 0..things as usize {
            let distance = inp();
            if distance < nearest { nearest = distance; winner = t as i16 + 1; }
        }
        out(winner);
    }
}
