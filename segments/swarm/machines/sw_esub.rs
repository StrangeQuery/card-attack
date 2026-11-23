//! Edges, for the swarm's polygons: an edge's run and height, its top end first. Deck: the count, then each edge's
//! ends (x, y; longs, hundredths of a pixel from the middle, y down), the top end first. Punches for each the run (x
//! below less x above) and the height (longs), and how far below the top end the first row's middle is (0..99:
//! rows' middles are at 100 cy + 50).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let (xh, xl, yh, yl) = (inp(), inp(), inp(), inp());
        // the rests differ by under 1000, borrowing a thousand when below 0
        let mut rh = inp() - xh;
        let mut rl = inp() - xl;
        if rl < 0 { rl += 999; rl += 1; rh -= 1; }
        out(rh); out(rl);
        let mut hh = inp() - yh;
        let mut hl = inp() - yl;
        if hl < 0 { hl += 999; hl += 1; hh -= 1; }
        out(hh); out(hl);
        // SAFETY: a long's rest is 0..999
        unsafe { assume(yl >= 0 && yl <= 999); }
        let mut first = 50 - (yl - digit(yl, 0) * 100);
        if first < 0 { first += 100; }
        out(first);
    }
}
