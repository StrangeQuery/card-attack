//! Compose, once per row per frame. Deck: where the ball's span starts and how many pixels it covers, the ball's
//! colour codes for those pixels, then the row's 96 background codes. Punches the row with the ball over it. In two
//! phases: the first reads the ball's codes (the host deals the deck: start, count and codes to the first).
// cardiac: replaces ray/original/compose.asm (in two phases)
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut ball = [0i16; 29];        // at most 29 pixels: radius 14
    let mut before = inp();            // background pixels before the ball
    let n = inp() as usize;            // the ball's pixels
    let mut i = 0;
    while i < n && i < 29 {
        ball[i] = inp();
        i += 1;
    }
    overlay();
    let mut next = 0;
    for _ in 0..96 {
        let background = inp();
        if before > 0 {
            before -= 1;
            out(background);
        } else if next < i {
            out(ball[next]);
            next += 1;
        } else {
            out(background);
        }
    }
}
