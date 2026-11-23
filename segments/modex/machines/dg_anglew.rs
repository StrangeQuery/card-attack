//! Angle from a picture's middle, for a picture of any size (the rotozoom's 192x96: rotodg), its middle on the deck.
//! Deck: the middle's doubled place across and down (w - 1, h - 1); how many; then each pixel's column u and row v. Its
//! middle is 2 u - (w - 1) across and 2 v - (h - 1) down from the frame's. Punches each: the quarter (0-3), then 1 if
//! the angle is past half the quarter (it is then a quarter less atan(near / far)) else 0, then near and far.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mx = inp();
    let my = inp();
    let count = inp();
    for _ in 0..count {
        let column = inp();
        let row = inp();
        let across = column + column - mx;
        let down = row + row - my;
        let mut quarter = 0;
        let mut near = abs(down);
        let mut far = abs(across);
        if across < 0 { quarter = 1; let t = near; near = far; far = t; }
        if down < 0 { quarter = 3 - quarter; let t = near; near = far; far = t; }
        out(quarter);
        if near > far { out(1); out(far); out(near); } else { out(0); out(near); out(far); }
    }
}
