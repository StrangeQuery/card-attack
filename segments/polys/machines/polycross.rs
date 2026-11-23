//! Cross, once per frame of the polys: the pixels a triangle covers in a row. Deck: for each of its rows, where its
//! two edges cross it (in quarter pixels, 0..767); then 999. A pixel's middle is at 4 x + 2; the triangle has the
//! pixels whose middles are from the left crossing to short of the right. Punches for each row: 1 if the second
//! crossing is the left (else 0), how many pixels, how far the first one's middle is from the left crossing, the
//! width; then each pixel's x.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let mut left = inp();
        if left == 999 { break; }
        let mut right = inp();
        if right < left {
            let t = left; left = right; right = t;
            out(1);
        } else {
            out(0);
        }
        // the first pixel whose middle is at or right of the left crossing: ten pixels at a time, then one
        let mut x = 0;
        let mut middle = 2;
        while middle + 40 < left { middle += 40; x += 10; }
        while middle < left { middle += 4; x += 1; }
        let mut pixels = 0;
        let mut at = middle;
        while at < right { at += 4; pixels += 1; }
        out(pixels);
        out(middle - left);
        out(right - left);
        for _ in 0..pixels {
            out(x);
            x += 1;
        }
    }
}
