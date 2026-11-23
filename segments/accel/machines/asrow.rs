//! Street row, once per row of the street per frame of the accelerated drive, for each of its numbers (distance,
//! across, lamps' light, haze; the dash's place is asdash's): each pixel's between the 25 sample columns (every 32
//! pixels; a pixel's middle i + 0.5 32nds of the way). Deck: for each of the 24 spans its two samples' numbers p and
//! q (|q - p| at most 999: within the haze's reach a span is under 12 units across). Punches for each of the 768
//! pixels p + (q - p) (2 i + 1) / 64, rounded toward p.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..24 {
        let p = inp();
        let gap = inp() - p;
        // |gap| (2 i + 1) / 64: |gap| / 64 to start, then |gap| / 32 a pixel, as a whole and 64ths
        let size = abs(gap);
        // size = 64 whole + part; size / 32 a pixel = 2 whole (+ 1 for a part of 32 or more), 2 (part mod 32) 64ths
        let mut whole = 0;
        let mut part = size;
        while part >= 64 { part -= 64; whole += 1; }
        let mut step = whole + whole;
        let mut half = part;
        if half >= 32 { half -= 32; step += 1; }
        half += half;
        for _ in 0..32 {
            out(if gap < 0 { p - whole } else { p + whole });
            whole += step;
            part += half;
            if part >= 64 { part -= 64; whole += 1; }
        }
    }
}
