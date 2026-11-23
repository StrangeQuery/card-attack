//! Rocky, once per screen row per frame of the dive. Deck: how many pixels of the row hit the seabed, then for each:
//! the face (0 a top, 1..3 a front or side, 4 the far seabed), the tone (4 coral) and the block's top (near, whole
//! units over the row's base) or how rocky it is (far, 32nds). Punches, per pixel, how rocky it is in 32nds (rock
//! against sand): a top's by its height, none at -2 units or below, all at 4.4 or above, 3 a unit between; a face
//! and coral all; far as dealt.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let face = inp();
        let tone = inp();
        let top = inp();
        let mut rocky = top;
        if face != 4 {
            rocky = 32;
            if face == 0 && tone < 4 {
                if top <= -2 { rocky = 0; }
                if top > -2 && top < 5 { rocky = (top * 10 + 20) * 5 / 10; }
            }
        }
        out(rocky);
    }
}
