//! Piece, per polygon per frame of the Rubik's cube: its spans cut into pieces of pixels. Deck: its colour (red,
//! green, blue, 64ths of a step), then its spans as rb_span punched them, then -1. Punches for each pixel a span
//! touches: the pixel's row (0-95) and column (0-191), the first and the last of its subsamples (0-3) the span
//! covers, and the colour.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// v / 2 and v mod 2, for v >= 0: twenty at a time, then two.
#[inline(never)]
fn halves(v: i16) -> (i16, i16) {
    let mut half = 0;
    let mut odd = v;
    while odd >= 20 { odd -= 20; half += 10; }
    while odd >= 2 { odd -= 2; half += 1; }
    (half, odd)
}

pub fn main() {
    let red = inp();
    let green = inp();
    let blue = inp();
    loop {
        let pixel_row = inp();
        if pixel_row < 0 { break; }
        let base = inp();               // the row's subsamples are base and base + 1
        let mut at = inp();
        let last = inp();
        let (mut pixel, mut side) = halves(at);   // the first's pixel column, and which of its two
        while at <= last {
            // to the pixel's right subsample, or the span's end
            let mut end = at + 1 - side;
            if end > last { end = last; }
            out(pixel_row);
            out(pixel);
            out(base + side);
            out(base + side + end - at);
            out(red);
            out(green);
            out(blue);
            at = end + 1;
            pixel += 1;
            side = 0;
        }
    }
}
