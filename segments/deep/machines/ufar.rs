//! Far, once per screen row per frame of the dive. Deck: how many pixels of the row hit the far seabed, then for each
//! the seabed's height there (tenths over its row's base). Punches, per pixel, the height as whole units rounded down
//! and hundredths 0..99.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let tenths = inp();
        let mut whole = tenths / 10;
        if whole * 10 > tenths { whole -= 1; }
        out(whole);
        out((tenths - whole * 10) * 10);
    }
}
