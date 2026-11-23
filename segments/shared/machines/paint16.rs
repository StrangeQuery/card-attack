//! Paint, 16 colours. Deck: the palette, 16 colours of red, green and blue, then one colour code (0-15) per pixel of
//! the row. Punches each pixel's red, green and blue. In two phases: the first reads the palette, the second paints
//! (the palette alone fills half of memory).
// cardiac: phase-cards=48
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// The palette's size.
const COLOURS: usize = 16;
/// Pixels in a row.
const WIDTH: i16 = 96;

pub fn main() {
    let mut palette = [Pixel::BLACK; COLOURS];
    for colour in 0..COLOURS {
        palette[colour] = read_pixel();
    }
    overlay();                          // phase 2: the painting loop loads over the palette loop
    for _ in 0..WIDTH {
        let code = inp();
        punch(palette[code as usize]);
    }
}
