//! Perspective, once per frame of the tumble's travelling cube. Deck: the tilt T (0-3), the cube's centre (X, Y)
//! across and down, its distance D from the eye, the lens L, then 8 corners as (x, y, depth). Depth shifts a corner
//! up and to the right by a = trunc(T depth / 10), as the projector does; then the corner is seen from the eye:
//! punches trunc(L (X + x + a) / (D + depth)), trunc(L (Y + y - a) / (D + depth)) and the depth, for the projector
//! (at tilt 0) to place.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// trunc(lens x size / distance) without the product, which would pass a card: add size lens times, taking the
/// distance away whenever the sum reaches it, so the sum stays below distance + size.
#[inline(never)]
fn seen(size: i16, lens: i16, distance: i16) -> i16 {
    let length = abs(size);
    let mut sum = 0;
    let mut seen = 0;
    for _ in 0..lens {
        sum += length;
        while sum >= distance { sum -= distance; seen += 1; }
    }
    if size < 0 { -seen } else { seen }
}

pub fn main() {
    let distance = inp();
    let lens = inp();
    for _ in 0..8 {                     // the corners
        let across = inp();
        let down = inp();
        let depth = inp();
        let far = distance + depth;     // further corners are smaller
        out(seen(across, lens, far));
        out(seen(down, lens, far));
        out(depth);
    }
}
