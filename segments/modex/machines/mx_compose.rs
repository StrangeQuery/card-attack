//! Compose, three times per tile row of the Mode X ray tracer (320x240): for the looks, the u and the v; and once
//! more on the rows of the sign, with 0 for the background and 1 for the ball, for where the ball covers the row.
//! Deck: where the ball's span starts in the tile's row and how many pixels it covers, then for each of the row's 96
//! pixels its background card and the ball's card there (any card where the ball isn't). Punches the row with the
//! ball over it.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut before = inp();            // background pixels before the ball
    let mut ball = inp();              // the ball's pixels
    for _ in 0..96 {
        let background = inp();
        let on_ball = inp();
        if before > 0 {
            before -= 1;
            out(background);
        } else if ball > 0 {
            ball -= 1;
            out(on_ball);
        } else {
            out(background);
        }
    }
}
