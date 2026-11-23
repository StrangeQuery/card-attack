//! Big word, once per screen row per frame a word is up. Deck: the fade (0..16), the screen row y, then for each of
//! the 96 pixels the word's bitmap code (0 none, 1 letter, 2 its black rim) and the picture's red, green and blue.
//! Where the word has something and the fade beats the pixel's place in a 4 x 4 ordered dither, the word shows:
//! white letters rimmed in black. Punches the row's 288 values.
// cardiac: replaces overlays/original/big.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let dither = tables::BAYER4;
    let fade = inp();
    let y = inp();
    // SAFETY: the deck's row is a screen row, 0..47 (0..95 in the cut, at 192x96)
    unsafe { assume(y >= 0) };
    let row_start = remainder(y, 4) * 4;   // this row's four thresholds start at 4 (y mod 4)
    let mut column = 0;                 // x mod 4
    for _pixel in 0..96 {
        let code = inp();               // 0 nothing of the word, 1 a letter, 2 its rim
        let picture = read_pixel();
        if code >= 1 && dither[(row_start + column) as usize] < fade {
            if code >= 2 {
                punch(Pixel { r: 0, g: 0, b: 0 });         // the rim: black
            } else {
                punch(Pixel { r: 250, g: 250, b: 250 });   // the letter: white
            }
        } else {
            punch(picture);
        }
        column += 1;
        if column >= 4 { column = 0; }
    }
}
