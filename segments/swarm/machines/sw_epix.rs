//! Pixels, for the swarm's polygons: an edge's crossing of a row (a long, hundredths of a pixel from the middle) as the
//! first pixel whose middle (100 x + 50) is at or right of it, held to -768..768. Deck: the count, then the crossings.
//! Punches each pixel.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let hi = inp();
        let lo = inp();
        let mut pixel = 768;
        if hi < 77 {
            pixel = -768;
            if hi > -78 {
                // SAFETY: a long's rest is 0..999
                unsafe { assume(lo >= 0 && lo <= 999); }
                // (x + 49) / 100: the thousands' tens and the rest's hundreds, one more past 50 in the hundred
                pixel = hi * 10 + digit(lo, 0);
                if lo - digit(lo, 0) * 100 > 50 { pixel += 1; }
                if pixel < -768 { pixel = -768; }
                if pixel > 768 { pixel = 768; }
            }
        }
        out(pixel);
    }
}
