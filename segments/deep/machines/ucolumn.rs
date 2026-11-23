//! Column, once per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed (made of
//! blocks a unit square, whole units high; near, the ray is always well within the map), then for each: the ray's
//! height a slice before the hit (whole units over the hit's row base), its x a slice before and at the hit (whole
//! units), its x a slice before in hundredths 0..99, its step across per slice (hundredths), and the block of the
//! hit's row at the x a slice before. Punches, per pixel: 1 when the ray was already under that block's top a slice
//! before (it went in through the block's front), 0 when it stayed in the column, 2 when it crossed into the next, 3
//! when it crossed more than one; and how many tenths of the slice it took to the first edge it crossed (at most 9).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let before = inp();
        let from = inp();
        let which = inp() - from;
        let x_part = inp();
        let across = inp();
        let here = inp();
        let mut mode = 0;
        let mut edge = 0;
        if before < here {
            mode = 1;
        } else if which != 0 {
            mode = 2;
            if which > 1 || which < -1 { mode = 3; }
            let mut gap = x_part;
            let mut size = across;
            if across > 0 { gap = 100 - x_part; } else { size = -across; }
            if gap > 99 { gap = 99; }
            let mut rest = gap * 10;
            while rest >= size { rest -= size; edge += 1; }
            if edge > 9 { edge = 9; }
        }
        out(mode);
        out(edge);
    }
}
