//! Vertex machine, once per frame of the turning cube. Two phases, each with its own cards.
//! First: the angle index A (0-59), the axis (0 vertical, 1 horizontal, 2 the view axis), then the 60-entry
//! sine table; keeps sin A and cos A = sin(A + 15). Second: each of the 8 corners in the rotation plane's order: the
//! signs sp, sq of the two coordinates that turn, then the coordinate that stays, sr (+-12, the cube's half-width).
//! p' = sp C + sq S, q' = sq C - sp S. The axis says which of p', q', sr is x, y and depth; punches (x, y, depth) per
//! corner for the projector. Corners are at +-1, so every product is +-1: additions and subtractions only.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let angle = inp();
    let axis = inp();
    let mut quarter_on = angle + 15;    // cos A is sin(A + 15), mod 60
    if quarter_on >= 60 { quarter_on -= 60; }
    // the table arrives a card at a time: keep the two entries we need as they go by
    let mut sin_a = 0;
    let mut cos_a = 0;
    for i in 0..60 {
        let entry = inp();
        if i == angle { sin_a = entry; }
        if i == quarter_on { cos_a = entry; }
    }
    overlay();                          // phase 2: the corners
    for _ in 0..8 {
        let sp = inp();                 // the corner's signs: in the plane (sp, sq), and along the axis (sr)
        let sq = inp();
        let sr = inp();
        // rotate (sp, sq) by A: p' = sp C + sq S, q' = sq C - sp S, where the signs only flip C and S
        let (sp_cos, sp_sin) = if sp < 0 { (-cos_a, -sin_a) } else { (cos_a, sin_a) };
        let p = sp_cos + if sq < 0 { -sin_a } else { sin_a };
        let q = if sq < 0 { -cos_a } else { cos_a } - sp_sin;
        match axis {
            ..=0 => { out(p); out(sr); out(q); }          // vertical axis: x = p', y = sr, depth = q'
            1 => { out(sr); out(p); out(q); }             // horizontal axis: x = sr, y = p', depth = q'
            _ => { out(p); out(q); out(sr); }             // view axis: x = p', y = q', depth = sr
        }
    }
}
