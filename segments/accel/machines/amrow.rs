//! Rows, once per sample column per frame of the accelerated drive: at each step of the march, the screen row where
//! the street is seen. Deck: the horizon's row, the lowest and highest rows punched, how many steps, then each: rows (220 D / a, rounded down; D the
//! camera's height over the street there, eighths), 16ths (16 r / a for what that left), and 16ths more in two
//! (440 F / a as 4 (110 F / a) + 4 r' / a, F the 64ths over). Punches each the row and its 16ths (0-15):
//! 16 H0 - 8 + 16 rows + 16ths, kept to the lowest and highest rows (each then with 0 16ths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let horizon = inp();
    let lowest = inp();
    let highest = inp();
    let steps = inp();
    for _ in 0..steps {
        // (far past the highest, it stays past it: the sum would pass a card)
        let q = inp();
        let mut row = if q > highest + 40 - horizon { highest + 40 } else { horizon - 1 + q };
        let mut frac = 8 + inp();
        let quarter = inp();
        // the four quarters of 440 F / a, each carried into rows as it comes (four of them would pass a card)
        for k in 0..5 {
            frac += if k < 4 { quarter } else { inp() };
            while frac >= 16 { frac -= 16; row += 1; }
            while frac < 0 { frac += 16; row -= 1; }
        }
        if row > highest - 1 { row = highest; frac = 0; }
        if row < lowest { row = lowest; frac = 0; }
        out(row);
        out(frac);
    }
}
