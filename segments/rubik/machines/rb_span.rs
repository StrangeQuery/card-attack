//! Span, per polygon per frame of the Rubik's cube: the subsamples it covers in each row. Deck: the bounds rb_walk
//! punched (row, bound, column), sorted by row and then bound (the host sorts them as a card sorter would), then
//! -1. A row's span runs from its rightmost left bound to its leftmost right bound, both included. A pixel is 2 x
//! 2 subsamples, numbered 0, 1 (its upper row, left and right) and 2, 3 (its lower). Punches each row that has both
//! bounds and a span not empty: the pixel row (0-95) and the number of the row's left subsample (0 or 2); the
//! span's first and last column (0-383). Then -1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// v / 2 and v mod 2, for v >= 0: twenty at a time, then two.
#[inline(never)]
fn halves(v: i16) -> (i16, i16) {
    let mut half = 0;
    let mut odd = v;
    while odd >= 20 { odd -= 20; half += 10; }
    while odd >= 2 { odd -= 2; half += 1; }
    (half, odd)
}

pub fn main() {
    let mut row = inp();
    while row >= 0 {
        let this = row;
        let mut first = -1;             // none yet
        let mut last = 999;             // none yet
        while row == this {
            let bound = inp();
            let column = inp();
            if bound > 0 {
                if column < last { last = column; }
            } else if column > first {
                first = column;
            }
            row = inp();
        }
        if first >= 0 && last < 999 && first <= last {
            let (pixel_row, lower) = halves(this);
            out(pixel_row);
            out(lower + lower);
            out(first);
            out(last);
        }
    }
    out(-1);
}
