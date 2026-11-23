//! Wall rows, once per wall a column's ray meets per frame of the accelerated drive: where up the wall each row is,
//! from the lowest open row up. Deck: how many walls, then each: Q (twentieths of a texel a half row: its whole, its
//! part over d, and d; a row is 2 Q), the open row, F there (texel, twentieths, and the part over d less d: -d to -1;
//! awalk's), the block's height (half units). Punches each: for every row from the open one up while F is at most the
//! top (texel 2 top - 1 and 10 twentieths) and the row is on the screen, 1, its texel and twentieths (rounded down)
//! and the texel above it (at most the top's); then 0 and the row above the last (the lowest row still open for the
//! walls behind).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let whole = inp();
        let part = inp();
        let d = inp();
        let mut y = inp();
        let mut texel = inp();
        let mut tw = inp();
        // the part over d kept less d (-d to -1), so that a part added never passes a card
        let mut rest = inp();
        let top = inp() * 2 - 1;
        while y >= 0 && (texel < top || (texel == top && tw <= 10)) {
            out(1);
            out(texel);
            out(tw);
            out(if texel < top { texel + 1 } else { top });
            y -= 1;
            for _ in 0..2 {
                tw += whole;
                rest += part;
                if rest >= 0 { rest -= d; tw += 1; }
            }
            while tw >= 20 { tw -= 20; texel += 1; }
        }
        out(0);
        out(y);
    }
}
