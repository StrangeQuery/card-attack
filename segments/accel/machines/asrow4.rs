//! Street row spans, the software half's, once per fourth row of the street per frame of the drive, for each of
//! asrow's numbers: each span made ready for its 4 x 4 blocks' middles (pixels 2, 6, .. 30). Deck: as asrow's.
//! Punches for each of the 24 spans p, the way (-1 or 1), the start 5 |q - p| / 64 and the step 8 |q - p| / 64,
//! each a whole and 64ths (asrun's).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..24 {
        let p = inp();
        let gap = inp() - p;
        out(p);
        out(if gap < 0 { -1 } else { 1 });
        let mut w = 0;
        let mut r = abs(gap);
        while r >= 64 { r -= 64; w += 1; }
        let (mut whole, mut part, mut sw, mut sp) = (0, 0, 0, 0);
        for k in 0..8 {
            if k < 5 { whole += w; part += r; if part >= 64 { part -= 64; whole += 1; } }
            sw += w;
            sp += r;
            if sp >= 64 { sp -= 64; sw += 1; }
        }
        out(whole);
        out(part);
        out(sw);
        out(sp);
    }
}
