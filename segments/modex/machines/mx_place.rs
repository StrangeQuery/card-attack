//! Place, once per frame of the Mode X ray tracer (320x240). Deck: the ball's x (floor units, -28..28) and height
//! (14 + its lift over the floor), as the ray's path machine punches them. At 320x240 a 192x96 pixel is 5 / 3 of
//! these, so a step of the ball's (two of 192x96's) is 10 / 3. Punches the ball's centre across, 160 + 10 x / 3,
//! and down, 107 - 10 L / 3 (L its lift), its centre's height over the floor, 46 + 10 L / 3, and its offset
//! across from the middle, 10 x / 3, all rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// 10 v / 3 rounded down, v -99..99: threes counted by taking them away (or adding them back, below 0).
fn ten_thirds(v: i16) -> i16 {
    let mut rest = 10 * v;
    let mut thirds = 0;
    while rest >= 3 { rest -= 3; thirds += 1; }
    while rest < 0 { rest += 3; thirds -= 1; }
    thirds
}

pub fn main() {
    let ball_x = inp();
    let lift = inp() - 14;
    let across = ten_thirds(ball_x);
    let up = ten_thirds(lift);
    out(160 + across);
    out(107 - up);
    out(46 + up);
    out(across);
}
