//! Car corners, once per car drawn per frame of the accelerated drive. Deck: how many, then each: its middle across
//! and its foot's row (each a pixel and 16ths), its width and height (pixels and 16ths). Punches each its left x - w /
//! 2 and top foot - h (each a pixel and 16ths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut x = inp();
        let mut xf = inp();
        let mut y = inp();
        let mut yf = inp();
        let w = inp();
        // half of 16 w + wf is 8 w + wf / 2
        let half = w / 2;
        xf -= inp() / 2 + 8 * (w - half - half);
        x -= half;
        while xf < 0 { xf += 16; x -= 1; }
        y -= inp();
        yf -= inp();
        while yf < 0 { yf += 16; y -= 1; }
        out(x);
        out(xf);
        out(y);
        out(yf);
    }
}
