//! Projector for one tile of the cube at twice the resolution (192x96, four 96x48 tiles), once per tile per frame.
//! Deck: the centre (cx, cy) on the 192x96 screen, the tile's place in it (tx 0 or 96 across, ty 0 or 48 down), the
//! tilt T (0-3), then 8 corners as (x, y, depth). As the projector: depth shifts a corner up and to the right by
//! a = trunc(T depth / 10). Punches each corner in the tile's own pixels: x = cx - tx + x + a, y = cy - ty + y - a.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let centre_x = inp();
    let centre_y = inp();
    let tile_x = inp();                 // 0 or 96 across ...
    let tile_y = inp();                 // ... 0 or 48 down
    let tilt = inp();                   // 0..3
    let (left, top) = (centre_x - tile_x, centre_y - tile_y);   // the centre in the tile's pixels
    for _ in 0..8 {                     // the corners
        let x = inp();
        let y = inp();
        let depth = inp();
        // the shift, trunc(tilt x depth / 10): the product by adding, since CARDIAC has no multiply
        let mut shift = 0;
        for _ in 0..tilt { shift += depth; }
        let shift = shift / 10;
        out(left + x + shift);
        out(top + y - shift);
    }
}
