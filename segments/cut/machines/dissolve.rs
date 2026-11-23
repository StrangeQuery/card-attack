//! Dissolve, once per screen row per frame of a transition. Deck: the level (0..16), the screen row y, then for each
//! of the 96 pixels the outgoing picture's red, green and blue and the incoming one's. Where the level beats the
//! pixel's place in a 4 x 4 ordered dither the incoming pixel shows, else the outgoing: one dissolves into the other
//! as the level rises. Punches the row's 288 values.
// cardiac: replaces cut/original/mix.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let dither = tables::BAYER4;
    let level = inp();
    let y = inp();
    // SAFETY: the deck's row is a screen row, 0..47 (0..95 in the cut, at 192x96)
    unsafe { assume(y >= 0) };
    let row_start = remainder(y, 4) * 4;   // this row's four thresholds start at 4 (y mod 4)
    let mut column = 0;                 // x mod 4
    for _pixel in 0..96 {
        let outgoing = read_pixel();
        let incoming = read_pixel();
        if dither[(row_start + column) as usize] < level { punch(incoming); } else { punch(outgoing); }
        column += 1;
        if column >= 4 { column = 0; }
    }
}
