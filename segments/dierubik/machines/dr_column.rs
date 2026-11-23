//! Column, per polygon per frame of the die and the Rubik's cube: the column each edge's walk begins at, as the
//! rubik segment's rb_column, about a middle column the frame chooses (the cube's, so the corners' x stay within a
//! card when the cube roams to the side). Deck: that column c, and 2 c - 1; then edges as rb_start punched them (their x from
//! column c's left side), then -1. Column xs has its middle at 8 (xs - c) + 4, and a mirrored edge's (x taken to -x)
//! column xs is 2 c - 1 - xs. Walking, a bound is found as the column before the first whose middle is right of the
//! edge; one that must take a middle exactly on the edge (a left bound, or a mirrored right one) starts its fraction
//! a hair (1 / down of an 8th) to the left. Punches for each edge: the bound, the first row, below, above, the
//! whole, the rest and down less the rest (passed on); the column bound at the top, which way it goes as the column
//! moves right (1, or -1 mirrored), how far that column's middle is right of the top (1-8) and the fraction's rest to
//! start with. Then -1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let middle = inp();
    let mirror = inp();
    loop {
        let bound = inp();
        out(bound);
        if bound < 0 { break; }
        let mirrored = inp();
        let x_top = inp();
        let hair = inp();               // down less 1
        for _ in 0..6 { out(inp()); }   // the first row, below, above, the whole, the rest, down less the rest
        let mut ahead = 4 - x_top;      // the middle column's middle less the top
        let mut rest = 0;
        let taken = bound != mirrored;  // a right bound as walked: the column before is the one
        if !taken { ahead += 1; rest = hair; }
        // the first column whose middle is right of the top (ahead 1-8)
        let mut column = middle;
        while ahead < 1 { ahead += 8; column += 1; }
        while ahead > 8 { ahead -= 8; column -= 1; }
        if taken { column -= 1; }
        let mut way = 1;
        if mirrored > 0 { column = mirror - column; way = -1; }
        out(column);
        out(way);
        out(ahead);
        out(rest);
    }
}
