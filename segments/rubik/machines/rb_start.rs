//! Start, per polygon per frame of the Rubik's cube: the rows each edge meets, and its slope. Deck: edges as
//! rb_edge punched them (bound, mirror, top x, top y, across, down), then -1. Row ys has its middle at 8 ys - 740
//! (8ths of a subsample from the screen's middle, y down); an edge meets the rows whose middles are from its top
//! (included) to its bottom (left out). Punches for each edge: the bound, the mirror and the top x (passed on),
//! and down less 1; the first row at or below the top, how far its middle is below the top and above the bottom
//! (8ths); the slope across / down as a whole number of 8ths, its rest, and down less the rest. Then -1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let bound = inp();
        out(bound);
        if bound < 0 { break; }
        out(inp());                     // the mirror
        out(inp());                     // the top x
        let y_top = inp();
        let across = inp();
        let down = inp();
        out(down - 1);                  // the hair: just short of a whole 8th
        // the first row at or below the top: rows are 8 apart, row 93's middle at 4
        let mut row = 93;
        let mut below = 4 - y_top;      // the row's middle less the top
        while below < 0 { below += 8; row += 1; }
        while below >= 8 { below -= 8; row -= 1; }
        out(row);
        out(below);
        out(down - below);
        // the slope, by repeated subtraction
        let mut whole = 0;
        let mut part = across;
        while part >= down { part -= down; whole += 1; }
        out(whole);
        out(part);
        out(down - part);
    }
}
