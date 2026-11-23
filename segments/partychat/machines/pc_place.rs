//! Places, once per frame of the party chat: where each line of each shown message is drawn, its shadow's
//! three black copies first. Deck: how many lines; then each: its left x, its message's top y, and which line of the
//! message it is (j, from 0). Punches, for each, the four places (x, y each): the shadow at (2, 2), (1, 2), (2, 1)
//! from the line's own place, then the line's, x and y + 29 j (layout.rs's SHADOW and LINE).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

const LINE: i16 = 29;

pub fn main() {
    let lines = inp();
    for _ in 0..lines {
        let x = inp();
        let mut y = inp();
        let j = inp();
        for _ in 0..j { y += LINE; }
        out(x + 2);
        out(y + 2);
        out(x + 1);
        out(y + 2);
        out(x + 2);
        out(y + 1);
        out(x);
        out(y);
    }
}
