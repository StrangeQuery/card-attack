//! Inside, per pixel of a bell point's box per frame of the dive. Deck: the point's place on the screen from the
//! middle (tenths of a pixel, across and up) and its radius (tenths, 5..31), how many pixels, then for each its
//! column and row counted from the middle's pixel (rows down). Punches, per pixel, 1 if its middle lies within the
//! radius of the point, else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let across = inp();
    let up = inp();
    let radius = inp();
    let count = inp();
    let mut limit = 0;
    for _ in 0..radius { limit += radius; }
    for _ in 0..count {
        let dx = abs(inp() * 10 + 5 - across);
        let dy = abs(inp() * 10 + 5 + up);
        let mut inside = 0;
        if dx <= radius && dy <= radius {
            // dx^2 <= r^2 - dy^2: each square under 1000
            let mut room = limit;
            for _ in 0..dy { room -= dy; }
            let mut square = 0;
            for _ in 0..dx { square += dx; }
            if square <= room { inside = 1; }
        }
        out(inside);
    }
}
