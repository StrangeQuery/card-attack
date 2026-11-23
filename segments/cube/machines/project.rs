//! Projector. Deck: the centre (cx, cy), the tilt T (0-3), then 8 corners as (x, y, depth). Depth shifts a corner up
//! and to the right by a = trunc(T depth / 10), so we see its front, top and right side: punches screen
//! x = cx + x + a and screen y = cy + y - a. T = 0 is the flat square.
// cardiac: replaces cube/original/project.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let centre_x = inp();
    let centre_y = inp();
    let tilt = inp();                   // 0..3
    for _ in 0..8 {                     // the corners
        let x = inp();
        let y = inp();
        let depth = inp();
        // the shift, trunc(tilt x depth / 10): the product by adding, since CARDIAC has no multiply
        let mut shift = 0;
        for _ in 0..tilt { shift += depth; }
        let shift = shift / 10;
        out(centre_x + x + shift);
        out(centre_y + y - shift);
    }
}
