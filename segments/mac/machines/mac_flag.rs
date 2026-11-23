//! Flag, once for Learn to Speak Finnish: the Finnish flag's pixels, row by row: its cross (the band down and the band
//! across) blue as three dots in four, every even row and the odd columns of the odd rows; the rest white (its edge is
//! the frame the host lays round it). Deck: its width and height, the band down's first column and the column after
//! it, the band across's first row and the row after it. Punches each pixel's dot (1 black, 0 white), row by row.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let width = inp();
    let height = inp();
    let x0 = inp();
    let x1 = inp();
    let y0 = inp();
    let y1 = inp();
    // the band down: its columns' count before it and in it; the band across: its rows' likewise
    let mut odd = 0;
    let mut row = 0;
    for _ in 0..height {
        // blue across the whole row on the bar, else only in the band down
        let mut across = 0;
        if row >= y0 { if row < y1 { across = 1; } }
        let mut blue = 1 - odd;
        let mut column = 0;
        for _ in 0..width {
            let mut dot = across;
            if column >= x0 { if column < x1 { dot = 1; } }
            if blue == 0 { dot = 0; }
            out(dot);
            blue += odd;
            if blue == 2 { blue = 0; }
            column += 1;
        }
        odd = 1 - odd;
        row += 1;
    }
}
