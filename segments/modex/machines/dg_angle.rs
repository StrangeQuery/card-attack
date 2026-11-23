//! Angle, once per row of the 320x240 frame (once for the whole degauss), before the atan machine (dg_atan): which
//! quarter of the turn each pixel's middle is in from the frame's middle, and the two sides whose ratio gives the
//! angle within it. Deck: how many; then each pixel's column u and row v. Its middle is 2 u - 319 across and
//! 2 v - 239 down from the frame's (in 1.6-pixel steps), the angle measured from across toward down. Punches each:
//! the quarter (0-3), then 1 if the angle is past half the quarter (it is then a quarter less atan(near / far))
//! else 0, then near and far (the smaller side and the larger).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let column = inp();
        let row = inp();
        let across = column + column - 319;
        let down = row + row - 239;
        // atan(down / across) in the first quarter; turning a quarter on swaps the sides
        let mut quarter = 0;
        let mut near = abs(down);
        let mut far = abs(across);
        if across < 0 { quarter = 1; let t = near; near = far; far = t; }
        if down < 0 { quarter = 3 - quarter; let t = near; near = far; far = t; }
        out(quarter);
        if near > far { out(1); out(far); out(near); } else { out(0); out(near); out(far); }
    }
}
