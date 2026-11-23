//! Corner, once for each radius a UI layer's boxes use: how much of each pixel of a rounded corner the box covers.
//! Deck: the radius r; then for each row of pixels from the circle's middle out, the circle machine's four rows of
//! 4ths (each reaching no further than the one before). Punches, for each row: how many pixels from the middle are
//! wholly covered; then each partly covered pixel's cover after them (1-31, in 32nds: 2 for each of its 16 4ths
//! inside), and 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let radius = inp();
    let mut reach = [0i16; 4];
    for _ in 0..radius {
        for k in 0..4 { reach[k] = inp(); }
        // whole pixels while the last row of 4ths, the shortest, covers all four
        let mut left = 0;
        let mut whole = 0;
        while reach[3] - left >= 4 {
            left += 4;
            whole += 1;
        }
        out(whole);
        // then part pixels while the first, the longest, reaches into them
        while reach[0] > left {
            let mut cover = 0;
            for k in 0..4 {
                let n = reach[k] - left;
                if n >= 4 { cover += 4; } else if n > 0 { cover += n; }
            }
            out(cover + cover);
            left += 4;
        }
        out(0);
    }
}
