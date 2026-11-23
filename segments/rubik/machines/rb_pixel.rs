//! Pixel, three times per pixel row per frame of the Rubik's cube (red, green, blue), for the pixels pieces fall
//! on; and once for the whole background before the first frame. Deck: the number of pixels n (1-192), then for
//! each pixel the background's value for this channel (0..255, from exact), then its pieces in drawing order (the
//! first and last subsample, 0-3, and the value, 0..255), then -1. Each piece paints its subsamples over what was
//! there; a subsample nothing paints shows the background. The pixel is the four subsamples' mean, rounded: their
//! sum and 2, over 4, the 4s carried as each subsample is added so the sum never passes a card. Punches each
//! pixel's value, 0..255.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let n = inp();
    for _ in 0..n {
        let background = inp();
        let mut sub = [background; 4];
        loop {
            let first = inp();
            if first < 0 { break; }
            let last = inp();
            let value = inp();
            // SAFETY: a piece's subsamples are 0..3, the first no later than the last
            unsafe { assume(first <= last && last <= 3) };
            for s in first..=last { sub[s as usize] = value; }
        }
        let mut mean = 0;
        let mut rest = 2;               // quarters: half of one, to round
        for s in 0..4 {
            rest += sub[s];
            while rest >= 40 { rest -= 40; mean += 10; }
            while rest >= 4 { rest -= 4; mean += 1; }
        }
        out(mean);
    }
}
