//! Cover: lays the cube over a picture, once per screen row (96 pixels) per frame. Deck: the cube's palette, 9 colours
//! of red, green and blue (code 0, no cube here, has one too but it is never shown), then for each pixel the cube's
//! colour code (0-8) and the picture's red, green and blue. Punches the palette's colour where the cube is, the
//! picture's where it isn't.
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
        let picture = read_pixel();
        if code >= 1 { punch(palette[code as usize]); } else { punch(picture); }
    }
}
