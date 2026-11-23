//! Glint, once per screen row per frame of the dive. Deck: the light in tenths (0..10), how many pixels of the row
//! hit the seabed, then for each how far the caustic waves' sum is from cancelling (0..60) and how far the hit is
//! along the view. Where the sum nearly cancels the light gathers: (16 - |sum|)^2 / 10 8 / 10 in 32nds, by the
//! light, fading out by 50 along the view. Punches, per pixel, how much of the caustics' colour to mix in (32nds).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let light = inp();
    let count = inp();
    for _ in 0..count {
        let near = 16 - inp();
        let mut fade = inp() / 2;
        if fade > 25 { fade = 25; }
        let mut glint = 0;
        if near > 0 { glint = times(times(near, near) / 10 * 8 / 10, light) / 10; }
        out(times(glint, 25 - fade) * 4 / 100);
    }
}

/// a b for a, b >= 0 and a b < 1000: a added b times.
#[inline(never)]
fn times(a: i16, b: i16) -> i16 {
    let mut product = 0;
    for _ in 0..b { product += a; }
    product
}
