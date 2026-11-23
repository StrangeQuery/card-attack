//! The camera's turn, once per frame of the drive. Deck: the street's slope ahead (eighths across in 96), then the
//! turn table: for each slope 0-80 the cosine and sine of its angle, times 400. Punches the cosine and the sine,
//! the sine negative for a slope to the left, and the sine's negative.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let slope = inp();
    let size = abs(slope);
    let mut cosine = 0;
    let mut sine = 0;
    for i in 0..81 {
        let c = inp();
        let s = inp();
        if i == size { cosine = c; sine = s; }
    }
    out(cosine);
    if slope < 0 { sine = -sine; }
    out(sine);
    out(-sine);
}
