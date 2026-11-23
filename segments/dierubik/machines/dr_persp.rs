//! Perspective, per shape per frame of the die and the Rubik's cube: its corners seen from the eye. Deck: the number
//! of values n, then n triples: a corner's x or y, its depth (476..944; x and y within half the depth), and a shift
//! in 64ths of an 8th (for x, the middle column's place: the cube's, to the nearest 8 subsamples, so x stays within a
//! card when the cube roams to the side; for y, 0). Seen at 264 subsamples a unit at unit distance. Punches 2112 v /
//! depth (8ths of a subsample from the screen's middle, cut toward zero) less 64 times the shift; then its negative.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let n = inp();
    for _ in 0..n {
        let v = inp();
        let depth = inp();
        let shift = inp();
        // 33 |v| / depth: |v| added 33 times, the depth taken away whenever the rest would reach it, since the
        // product would pass a card
        let add = abs(v);
        let gap = depth - add;          // the rest at or past this carries
        let mut rest = 0;
        let mut seen = 0;
        for _ in 0..33 {
            if rest >= gap { rest -= gap; seen += 1; } else { rest += add; }
        }
        let mut way = 1;
        if v < 0 { seen = -seen; way = -1; }
        // the shift taken off in whole 64ths, before the doubling could pass a card
        seen -= shift;
        // six binary places by doubling the rest: 64ths, so 2112 v / depth
        for _ in 0..6 {
            seen += seen;
            if rest >= depth - rest { rest -= depth - rest; seen += way; } else { rest += rest; }
        }
        out(seen);
        out(-seen);
    }
}
