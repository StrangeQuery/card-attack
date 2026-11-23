//! Span row, once per steep leaf per frame of the dive. Deck: the leaf's start across (tenths of a pixel from the
//! middle), its reach, its first and last column, how many rows, then for each how far across from the start its
//! middle line is. Its pixels lie within reach + 10 of the line. Punches, per row, the first and last column to look
//! at: (start + line -/+ (reach + 10)) / 10 rounded down, kept within the leaf's columns (and without passing a
//! card: the start is split into tens and units first).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let start = inp();
    let reach = inp() + 10;
    let left = inp();
    let right = inp();
    let count = inp();
    let mut tens = start / 10;
    let mut units = start - tens * 10;
    if units < 0 { units += 10; tens -= 1; }
    for _ in 0..count {
        let line = inp();
        let mut side = -reach;
        for _ in 0..2 {
            // tens + (units + line + side) / 10, rounded down
            let mut t = tens;
            let mut v = units + line + side;
            while v < 0 { v += 10; t -= 1; }
            while v >= 10 { v -= 10; t += 1; }
            if t < left { t = left; }
            if t > right { t = right; }
            out(t);
            side = reach;
        }
    }
}
