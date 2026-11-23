//! Street dash spans, once per row of the street per frame of the accelerated drive: for each span between two sample
//! columns, the way from one's place in the dash (16ths, 0-479) to the next's, the nearer way round 480. Deck: the
//! 25 samples' places. Punches for each of the 24 spans: the first's place, the way (-1 or 1), and for a gap g that
//! way, |g| / 64 (whole and 64ths over) to start and |g| / 32 (whole and 64ths) a pixel.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut q = inp();
    for _ in 0..24 {
        let p = q;
        q = inp();
        let mut gap = q - p + 240;
        if gap < 0 { gap += 480; }
        if gap >= 480 { gap -= 480; }
        gap -= 240;
        out(p);
        out(if gap < 0 { -1 } else { 1 });
        // |g| = 64 whole + part; a pixel: 2 whole (+ 1 for a part of 32 or more), 2 (part mod 32) 64ths
        let mut whole = 0;
        let mut part = abs(gap);
        while part >= 64 { part -= 64; whole += 1; }
        out(whole);
        out(part);
        let mut half = part;
        let mut step = whole + whole;
        if half >= 32 { half -= 32; step += 1; }
        out(step);
        out(half + half);
    }
}
