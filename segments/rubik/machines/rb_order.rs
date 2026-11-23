//! Order, once per frame of the Rubik's cube: which cubies turn with the layer, and which half of the cube is
//! drawn first. Deck: the layer's side times the view's depth along its axis (999 scale; -999 when nothing turns),
//! the layer's side (-1 or 1; 0 when nothing turns), then for each of the 27 cubies its place along the layer's
//! axis (-1, 0 or 1); a cubie on the layer's side turns. The eye is on the layer's side when the first card is 56 or more (half a cubie out of the middle, seen
//! from 9 units: 999 / 18); the half the eye isn't on is drawn first. Punches for each cubie: 1 if it turns with
//! the layer, else 0; then 1 if it is drawn in the second half, else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let eye_side = inp() >= 56;
    let side = inp();
    for _ in 0..27 {
        let along = inp();
        let turning = (side > 0 && along > 0) || (side < 0 && along < 0);
        let mut group = 0;
        if turning { group = 1; }
        let mut later = 0;
        if turning == eye_side { later = 1; }
        out(group);
        out(later);
    }
}
