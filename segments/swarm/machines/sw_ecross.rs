//! Edge walks, the crossings: where an edge crosses each row on the screen. Deck: the count; then each edge: the
//! crossing at its first row and the step a row on (longs, hundredths of a pixel from the middle), the first row on
//! the screen and the one after the last (counted from the edge's first row), the first's screen row, then a card a
//! row (1 when the step takes a hundredth more). Punches for each row on the screen its row and the crossing (a long).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut xh = inp();
        let mut xl = inp();
        let sh = inp();
        let sl = inp();
        let on = inp();
        let end = inp();
        let mut row = inp();
        for k in 0..end {
            if k >= on { out(row); out(xh); out(xl); row += 1; }
            xh += sh;
            xl -= 999 - sl;
            if xl > 0 { xl -= 1; xh += 1; } else { xl += 999; }
            if inp() > 0 { if xl == 999 { xl = 0; xh += 1; } else { xl += 1; } }
        }
    }
}
