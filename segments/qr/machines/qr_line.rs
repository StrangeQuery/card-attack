//! The QR code's rows, once per module row of the code. Deck: how many modules a row, how many pixels a module, then
//! the row's modules (1 dark, 0 light). A dark module is white, a light one black (white on black).
//! Punches the row's pixels, a module's pixels each, red, green and blue.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let modules = inp();
    let size = inp();
    for _ in 0..modules {
        let mut v = 0;
        if inp() > 0 { v = 255; }
        for _ in 0..size {
            out(v);
            out(v);
            out(v);
        }
    }
}
