//! Paint, 9 colours. Deck: the palette, 9 colours of red, green and blue, then one colour code (0-8) per pixel of
//! the row. Punches each pixel's red, green and blue.
// cardiac: replaces shared/original/paint.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// The palette's size.
const COLOURS: usize = 9;
/// Pixels in a row.
const WIDTH: i16 = 96;

pub fn main() {
    let mut palette = [Pixel::BLACK; COLOURS];
    for colour in 0..COLOURS {
        palette[colour] = read_pixel();
    }
    for _ in 0..WIDTH {
        let code = inp();
        punch(palette[code as usize]);
    }
}
