//! Atan, once per row of the picture (once for the whole degauss), after the angle machine (dg_anglew): each
//! pixel's angle within its quarter. Deck: how many; then each: 1 if the angle is past half the quarter else 0, the
//! near side and the far (near at most far, far 1 or more), then the table 10 atan(k / 256) in degrees, k = 0..256,
//! read one card at a time. Punches each angle in tenths of a degree (0-899): the table at 256 near / far (rounded
//! down), or 900 less it.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let flip = inp();
        let near = inp();
        let far = inp();
        // 256 near / far: the whole part, then eight halving steps (the rest doubled, far taken off when it fits)
        let mut ratio = 0;
        let mut rest = near;
        if rest >= far { rest -= far; ratio = 1; }
        for _ in 0..8 {
            rest += rest;
            ratio += ratio;
            if rest >= far { rest -= far; ratio += 1; }
        }
        for _ in 0..ratio { inp(); }            // skip to it in the table
        let angle = inp();
        for _ in ratio..256 { inp(); }          // and past the rest
        if flip > 0 { out(900 - angle); } else { out(angle); }
    }
}
