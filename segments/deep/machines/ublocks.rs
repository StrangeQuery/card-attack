//! Blocks, a few times per pixel per frame of the dive where the near seabed is hit. Deck: x as a whole unit, which
//! column to start from (columns from x's), how many, then a row of the height map: 201 heights in tenths over its
//! base, x = -100..100. The near seabed is made of blocks a unit square, each as high as its column's height snapped
//! down to an even number of units. Punches, per column asked for (held to the map's edges), its block's height in
//! whole units over the base.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let x = inp();
    let first = inp();
    let count = inp();
    let mut column = x + 100 + first;
    let mut read = 0;                  // cards of the row read so far
    let mut height = 0;
    for _ in 0..count {
        let mut want = column;
        if want < 0 { want = 0; }
        if want > 200 { want = 200; }
        if read <= want {
            // the cards before the one wanted are read and dropped, ten at a time while there are ten
            let mut skip = want - read;
            read = want + 1;
            while skip >= 10 {
                skip -= 10;
                inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp();
            }
            for _ in 0..skip { inp(); }
            height = inp();
        }
        // 2 floor(height / 20): the whole units rounded down, less one if odd
        let mut pairs = height / 20;
        if height < 0 { pairs = -((19 - height) / 20); }
        out(pairs * 2);
        column += 1;
    }
}
