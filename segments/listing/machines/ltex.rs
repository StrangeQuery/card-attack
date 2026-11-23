//! Typesetter for the listing, once per listing line per glyph row. Deck: the glyph row g (0..4) and the number of
//! characters n, then for each character its five font cards (each a glyph row as three binary digits, 101 for
//! X.X). Punches the g-th font card of each character: one row of the line, n cards.
// cardiac: replaces listing/original/ltex.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let wanted = inp();                 // which of the five glyph rows
    let characters = inp();
    for _character in 0..characters {
        let mut kept = 0;
        for row in 0..5 {               // read all five font cards, keep the wanted one
            let card = inp();
            if row == wanted { kept = card; }
        }
        out(kept);
    }
}
