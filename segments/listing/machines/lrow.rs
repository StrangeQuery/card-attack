//! Listing row, once per screen row per frame. Its program deck carries the row of the listing it shows, 24 glyph
//! rows (three binary digits each) in `tex`, punched in by the host at the label resource_tex. Deck: the text colour code. Each character is 4 pixels, 3 of glyph and a gap. Punches 96 codes: the
//! colour where a glyph bit is set, else 0.
// cardiac: replaces listing/original/lrow.asm (the row patched in at resource_tex)
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let tex: [i16; 24] = load_resource("tex");  // the host punches the row in here (at the label resource_tex)
    let colour = inp();
    for character in 0..24 {
        for column in 0..3 {
            out(if digit(tex[character], column) >= 1 { colour } else { 0 });
        }
        out(0);                         // the gap between characters
    }
}
