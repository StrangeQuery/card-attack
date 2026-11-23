//! Rows, once per frame of the polys: the rows each triangle's edge meets. Deck: for each edge, its ends' heights
//! (in quarter pixels); then 999. A row's middle is at 4 y + 2; an edge meets the rows of the screen (0..95) whose
//! middles are from its upper end to short of its lower. Punches for each edge: 1 if its second end is the upper
//! (else 0), the first row, how many, how far the first row's middle is below the upper end, and the edge's height.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let mut upper = inp();
        if upper == 999 { break; }
        let mut lower = inp();
        if lower < upper {
            let t = upper; upper = lower; lower = t;
            out(1);
        } else {
            out(0);
        }
        // the first row whose middle is at or below the upper end: ten rows at a time, then one
        let mut row = 0;
        let mut middle = 2;
        while middle + 40 < upper { middle += 40; row += 10; }
        while middle < upper { middle += 4; row += 1; }
        out(row);
        let mut rows = 0;
        let mut at = middle;
        while at < lower {
            if row + rows > 95 { break; }
            at += 4;
            rows += 1;
        }
        out(rows);
        out(middle - upper);
        out(lower - upper);
    }
}
