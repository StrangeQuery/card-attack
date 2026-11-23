//! Listing row selector, once per screen row per frame of the boot cards. Deck: the scroll s (pixels) and the
//! screen row y. The row shows listing pixel row s + y: punches the listing line (s + y) / 6 and the glyph row
//! (s + y) mod 6.
// cardiac: replaces listing/original/lsel.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let scroll = inp();
    let y = inp();
    // line = (scroll + y) / 6 and glyph row = (scroll + y) mod 6, by subtraction
    let (line, glyph_row) = divide(scroll + y, 6);
    out(line);
    out(glyph_row);
}
