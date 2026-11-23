//! Circle, once for each radius a UI layer's boxes use: a quarter circle cut finely, for the corner machine. Deck:
//! the radius r (1-26). The circle has radius 4r (each pixel cut into 4 x 4) and is walked down its rows by the
//! midpoint method: only sums and comparisons, the walk's error staying within a card. Punches, for each of its 4r
//! rows from the middle out, how many 4ths across from the middle are inside it (their middles).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let big = inp() * 4;
    // across: the 4ths inside on this row; error: 4 big^2 - (2 across - 1)^2 - (2 row + 1)^2, kept at 0 or more
    // (the last one counted is inside) and below 8 across (the next one isn't)
    let mut across = big;
    let mut error = big * 4 - 2;
    for row in 0..big {
        if row > 0 {
            error -= row * 8;
            while error < 0 && across > 0 {
                across -= 1;
                error += across * 8;
            }
        }
        out(across);
    }
}
