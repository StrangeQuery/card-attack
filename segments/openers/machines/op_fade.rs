//! Opening cards fade, once per frame of the opening cards. Deck: the frame within its card (0 on), and how many
//! frames the card shows (the three of black after it count on past that). The card fades in over its first five
//! frames and out over its last five, by fifths: steps = min(5, frame + 1, frames - frame), none after the card.
//! Punches the steps (0-5) and the white's level, 51 a step (0-255).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let frame = inp();
    let frames = inp();
    let mut steps = 5;
    if frame + 1 < steps { steps = frame + 1; }
    if frames - frame < steps { steps = frames - frame; }
    if steps < 0 { steps = 0; }
    out(steps);
    // 51 a step, by adding: CARDIAC has no multiply
    let mut level = 0;
    while steps > 0 {
        level += 51;
        steps -= 1;
    }
    out(level);
}
