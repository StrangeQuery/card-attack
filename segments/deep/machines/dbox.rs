//! Box, once per jellyfish per frame of the dive. Deck: its middle's place on the screen from the screen's middle
//! (tenths of a pixel, across and up), 14000 / ahead and 3000 / ahead (ahead in tenths). Its halo is a fall-off of
//! width the first, at least 2.5 pixels, out to the second, at least 5 pixels. Punches the width (tenths) and the
//! columns and rows the halo covers, counted from the middle's pixel (rows down): first column, last column + 1,
//! first row, last row + 1 (the host deals only those on the screen).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let across = floor10(inp());
    let down = floor10(-inp());
    let mut width = inp();
    if width < 25 { width = 25; }
    let mut reach = inp();
    if reach < 5 { reach = 5; }
    out(width);
    out(across - reach);
    out(across + reach);
    out(down - reach);
    out(down + reach);
}

/// v / 10 rounded down.
#[inline(never)]
fn floor10(v: i16) -> i16 { let q = v / 10; if q * 10 > v { q - 1 } else { q } }
