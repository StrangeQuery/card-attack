//! Side, once per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed, then for
//! each: how the ray went (as the column machine punches it: 1 in through a front, 0 staying in its column, 2
//! crossing into the next, 3 crossing more), the tenths of the slice to the first edge it crossed, the tenths it took
//! to go down to the first column's top and to the last's, and its step across. Crossing one edge, it hit the first
//! column's top if it went under it before the edge, else the last column's side if it was under that top at the
//! edge, else its top; crossing more, the last column's top if it was over it a slice before, else its side halfway.
//! Punches, per pixel: the face it hit (0 a top, 1 a front, 2 a side facing -x, 3 a side facing +x), how many tenths
//! of the slice from the slice before it met it (0..9), and whose block it is (0 the first column's, 1 the last's).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mode = inp();
        let edge = inp();
        let first = inp();
        let mut tenths = inp();
        let across = inp();
        let mut face = 0;
        let mut last = 0;
        if mode == 1 { face = 1; tenths = 0; }
        if mode == 2 {
            if first < edge {
                tenths = first;
            } else {
                last = 1;
                if tenths < edge { tenths = edge; face = 3; }
            }
        }
        if mode == 3 {
            last = 1;
            if tenths == 0 { tenths = 5; face = 3; }
        }
        if face == 3 && across > 0 { face = 2; }
        out(face);
        out(tenths);
        out(last);
    }
}
