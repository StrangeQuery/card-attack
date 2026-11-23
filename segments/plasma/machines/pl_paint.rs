//! Paint, three times per tile row of the plasma at 192x96: once each for red, green and blue. Deck: for each of
//! the 96 pixels its place on the colour ring (the four waves and the colours' start added, 0..511). The ring is
//! eight colours, the host's ring every sixth line, 32 steps apart: this one's red, green or blue of each is punched
//! into the program, with the first again at the end. Punches each pixel's value exactly: the place mod 256, and
//! between the two colours either side of it, a + (b - a) j / 32 rounded down, j its step past a.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let keys: [i16; 9] = load_resource("keys");   // the host punches the eight colours' red, green or blue (and the first again) in here
    for _ in 0..96 {
        let mut j = inp();
        if j >= 256 { j -= 256; }
        let mut k = 0;                 // the colour before
        while j >= 32 {
            j -= 32;
            k += 1;
        }
        // SAFETY: k is a whole number of 32s in 0..255, so 0..7
        unsafe { assume(k <= 7); }
        let from = keys[k];
        // (b - a) j / 32 rounded down, a bit of j at a time from the lowest: add b - a if the bit is set, then
        // halve, five times; rounding down at each halving rounds the whole down. The part is kept 250 up, so
        // that every number halved is 0..999 (halving a negative number on CARDIAC rounds toward zero). Halving is
        // five times it, shifted right one digit.
        let step = keys[k + 1] - from + 250;
        let mut part = 250;
        j += 32;                       // a bit past j's five: the halving stops when only it is left
        loop {
            let half = j / 2;
            if half < 1 { break; }
            if j > half + half { part += step; } else { part += 250; }
            // SAFETY: before this step the part was 250 up from at most 15/16 of b - a (239 either way), and the
            // step adds 250 and perhaps b - a (255 either way): 6..994
            unsafe { assume(part >= 0); }
            part /= 2;
            j = half;
        }
        out(from + part - 250);
    }
}
