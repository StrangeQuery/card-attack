//! Wall meeting along a face, once per face a column's ray meets per frame of the accelerated drive. Deck: how many,
//! then each: the ray along and |across|, the face's |distance| across (eighths). Punches each X = |e| along /
//! |across| (twice the quarters ahead where the ray meets the face) as its half (the quarters) and what is over (0
//! or 1), and what the division leaves.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let along = inp();
        let across = inp();
        let e = inp();
        // along added |e| times, |across| taken away whenever the sum reaches it, counted in halves
        let mut met = 0;
        let mut half = 0;
        let mut sum = -across;
        for _ in 0..e {
            sum += along;
            // (met stops at 999, a card's most: a face that far ahead is past the fog)
            while sum >= 0 { sum -= across; half += 1; if half == 2 { half = 0; if met < 999 { met += 1; } } }
        }
        out(met);
        out(half);
        out(sum + across);
    }
}
