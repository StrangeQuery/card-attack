//! Text, once a frame of a UI layer, for all its lines of text and its sprites: where each glyph goes. Deck: how many
//! lines; then each: x and its group's move, y and its group's move and how far its top sits above the anchor,
//! how it sits across (0 starting at x, 1 centred on it, 2 ending at it), how many glyphs; for 1 and 2 each glyph's
//! advance (for the width); then each glyph's advance and its bitmap's place from the pen (across, down). Punches
//! each glyph's bitmap's place: x, y.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut pen = inp() + inp();
        let top = inp() + inp() - inp();
        let across = inp();
        let glyphs = inp();
        if across > 0 {
            let mut width = 0;
            for _ in 0..glyphs { width += inp(); }
            if across == 1 { width /= 2; }
            pen -= width;
        }
        for _ in 0..glyphs {
            let advance = inp();
            out(pen + inp());
            out(top + inp());
            pen += advance;
        }
    }
}
