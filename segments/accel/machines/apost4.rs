//! Post arm rows and lights, once per lamp post drawn per frame of the accelerated drive. Deck: how many, then each:
//! sc (quarters of a pixel a unit), the top's row and 16ths. Punches each the arm's rows, from the top's less sc / 32
//! (at least 1 row; at least 0) to the top's (at most 383), and the light's place down, the top's and sc / 4 16ths
//! (row, 16ths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let sc = inp();
        let mut rows = sc / 2 / 2 / 2 / 2 / 2;
        if rows < 1 { rows = 1; }
        let mut row = inp();
        let mut top = row - rows;
        if top < 0 { top = 0; }
        out(top);
        out(if row > 383 { 383 } else { row });
        let mut down = inp() + sc / 2 / 2;
        while down >= 16 { down -= 16; row += 1; }
        out(row);
        out(down);
    }
}
