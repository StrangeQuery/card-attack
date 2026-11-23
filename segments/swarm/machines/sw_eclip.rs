//! Edge rows above the screen, for the swarm's polygons. Deck: the count, then each edge's top y (a long, hundredths
//! of a pixel from the middle, down) and how many rows it meets. Punches for each how many of its rows are above the
//! screen (at most all of them; all of them when it starts below), and the screen row (-384..383) its first row on
//! the screen would be. Its first row is (y + 49) / 100 rounded down: the long's thousands are hundreds of rows, and
//! the rest's hundreds the rows.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut hundreds = inp();
        let mut rest = inp() - (999 - 49);
        if rest > 0 { rest -= 1; hundreds += 1; } else { rest += 999; }
        // SAFETY: a long's rest is 0..999
        unsafe { assume(rest >= 0 && rest <= 999); }
        let rows = inp();
        let rows_in = digit(rest, 0);
        let (mut above, mut row) = (rows, -384);
        if hundreds >= -39 {
            if hundreds < 39 {
                row = hundreds * 10 + rows_in;
                above = 0;
                if row < -384 { above = -384 - row; row = -384; }
            }
        } else if hundreds > -139 {
            // above the screen: the top row (-384) is -384 - (10 h + rows) rows on
            above = (-39 - hundreds) * 10 + 6 - rows_in;
        }
        if above > rows { above = rows; }
        out(above); out(row);
    }
}
