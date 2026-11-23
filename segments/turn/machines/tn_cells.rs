//! The screen's runs cut at its grid, for the turn: a run of pixels on a row in pieces, a cell of the grid each (the
//! grid's columns 20 pixels apart, the first at pixel -768). Deck: the count; then each run's first pixel and the one
//! after its last (-768..768). Punches for each piece its cell (0..76), how many of the cell's pixels it passes before
//! its first, how many it has, and 1 if it is the run's last (else 0).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut x = inp();
        let after = inp();
        // the cell of the first pixel: x + 768 is 20 cell + pass, and x + 8 is 20 (cell - 38) + pass, which never
        // passes a card
        let mut cell = 38;
        let mut pass = x + 8;
        while pass < 0 { pass += 20; cell -= 1; }
        while pass > 19 { pass -= 20; cell += 1; }
        let mut left = x - pass;
        loop {
            let next = left + 20;
            out(cell);
            out(x - left);
            if next >= after {
                out(after - x);
                out(1);
                break;
            }
            out(next - x);
            out(0);
            x = next;
            left = next;
            cell += 1;
        }
    }
}
