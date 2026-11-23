//! Cross, once per frame of Boom: each column's ray across the map's lines one way (the lines x = a cell's edge, or
//! y = one), a line at a time: the cell it enters and how far it has gone the other way, a remainder carried from line
//! to line as a line drawer does (the way across is |b| / |a| of the way along). Deck: the count of lists; each: how far
//! to the first line (texels), the cell entered there and the way to the next (1 or -1), how many cells the map has
//! this way; how far the ray has gone the other way at the first line (999 for past every wall), its remainder (over
//! |a|), |a|; how much further at each next line and its remainder. Punches each crossing, stopping at 999 texels
//! either way or the map's edge: the cell, how far across, 1 if the crossing is short of the next texel (a remainder)
//! else 0; then -1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut along = inp();
        let mut cell = inp();
        let way = inp();
        let cells = inp();
        let mut across = inp();
        let mut rem = inp();
        let a = inp();
        let more = inp();
        let r = inp();
        // 999 texels along or across is past every wall
        while along < 999 && across < 999 && cell >= 0 && cell < cells {
            out(cell);
            out(across);
            if rem > 0 { out(1); } else { out(0); }
            // the next line would be 999 or more along (and the sum would pass a card)
            if along >= 935 { break }
            along += 64;
            cell += way;
            // the remainders pass |a| (a texel more) just when rem is at least |a| less the remainder (the sum would
            // pass a card); the way across is held under 999 (999 is past every wall)
            let short = a - r;
            if rem >= short {
                rem -= short;
                across += 1;
            } else {
                rem += r;
            }
            if across >= 999 - more { across = 999; } else { across += more; }
        }
        out(-1);
    }
}
