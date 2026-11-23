//! Dither, once per row of a one-bit picture, top to bottom (the tunnel's blocks, each picture):
//! Bill Atkinson's dither from MacPaint. Each block is black under half (128 of 256); what it was off by, in eighths
//! (toward zero), goes to the next two along, the three under it and the one two under (3/4 of it passed on). The row
//! machine carries what the rows above passed down. Deck: how many blocks the row has; for each, its shade (256
//! white), what the rows above passed it, and what the row above passed to the block two under it (its eighth for the
//! row after this). Punches for each block: 1 black or 0 white, its eighth (for the row after next); and from the second
//! block on, everything passed to the block under the one before it (for the next row); after the last, the same for
//! the block under it.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    // the eighths of the two blocks before, and what the row above passed to the one before's block two under
    let mut before = 0;
    let mut two_before = 0;
    let mut under_before = 0;
    let width = inp();
    for x in 0..width {
        let shade = inp();
        let passed = inp();
        let under = inp();
        let value = shade + passed + before + two_before;
        let mut error = value;
        if value < 128 {
            out(1);
        } else {
            out(0);
            error -= 256;
        }
        // an eighth, toward zero, by halving three times
        let eighth = error / 2 / 2 / 2;
        out(eighth);
        if x > 0 { out(under_before + two_before + before + eighth); }
        two_before = before;
        before = eighth;
        under_before = under;
    }
    out(under_before + two_before + before);
}
