//! Ground rows' share, once per sample column per frame of the accelerated drive: for each row that sees the street
//! between two steps of the march, how far it is from the nearer step: n of d. Deck: how many rows, then each: the
//! nearer step's row and 16ths, the further step's row and 16ths, the row. Punches each n and d: in 16ths of a row
//! (R0 - 16 y of R0 - R1), or whole rows when the steps are 62 or more rows apart.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let row0 = inp();
        let frac0 = inp();
        let row = inp();
        let frac = inp();
        let y = inp();
        if row0 - row >= 62 {
            out(row0 - y);
            out(row0 - row);
        } else {
            out(16 * (row0 - y) + frac0);
            out(16 * (row0 - row) + frac0 - frac);
        }
    }
}
