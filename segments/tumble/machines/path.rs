//! Path: where the cube is as it travels, once per frame of the tumble. Two phases. First: three steps a, b and c
//! (0-119), then a 120-entry sine table scaled to 48 (3 degrees a step); keeps the three entries, and works out the drop. Second (no cards):
//! at half size, the cube's centre is sin a - trunc(sin a / 4) across (36 either side), trunc(sin b / 6) down (8
//! either side), and 180 + sin c + trunc(sin c / 2) from the eye (108 to 252). Punches those three, then twice each
//! for the whole cube, then the small cube's lens, its distance less 12. The host turns a, b and c at different
//! rates, so the path winds in all three directions.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let a = inp();
    let b = inp();
    let c = inp();
    // the table arrives a card at a time: keep the three entries we need as they go by
    let mut sin_a = 0;
    let mut sin_b = 0;
    let mut sin_c = 0;
    for i in 0..120 {
        let entry = inp();
        if i == a { sin_a = entry; }
        if i == b { sin_b = entry; }
        if i == c { sin_c = entry; }
    }
    let down = sin_b / 6;               // divisions toward zero; this one here, where there's room
    overlay();                          // phase 2 (no cards): the rest of the place
    let across = sin_a - sin_a / 4;
    let away = 180 + sin_c + sin_c / 2;
    out(across);
    out(down);
    out(away);
    out(across + across);               // twice each, by adding: the big cube is exactly twice the small one
    out(down + down);
    out(away + away);
    out(away - 12);                     // the small cube's lens: its near face, face on, at its own size
}
