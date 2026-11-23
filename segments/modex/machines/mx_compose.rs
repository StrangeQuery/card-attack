//! Compose, three times per tile row of the Mode X ray tracer (320x240): for the looks, the u and the v. Deck: where
//! the ball's span starts in the tile's row and how many pixels it covers, then the row's 96 background cards, each
//! of the ball's pixels followed by the ball's card. Punches the row with the ball over it.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut before = inp();            // background pixels before the ball
    let mut ball = inp();              // the ball's pixels
    for _ in 0..96 {
        let background = inp();
        if before > 0 {
            before -= 1;
            out(background);
        } else if ball > 0 {
            ball -= 1;
            out(inp());
        } else {
            out(background);
        }
    }
}
