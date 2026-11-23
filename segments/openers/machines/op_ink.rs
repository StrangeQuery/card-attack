//! Opening cards ink, once per screen row per frame of the opening cards. Deck: the white's level (op_fade, 0-255),
//! then the row's 192 pixels of the card's bitmap (1 a letter, 0 not). A letter is grey at the level, the rest
//! black. Punches the row's 192 colours, red, green and blue.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let level = inp();
    for _ in 0..192 {
        let mut v = 0;
        if inp() > 0 { v = level; }
        out(v);
        out(v);
        out(v);
    }
}
