//! The wobble in the camera's path and our car's lane, 0.4 sin(1.7 t): its phase each frame, as a place in the
//! quarter-wave table (data/lanesine.txt, 251 entries) and the sine's sign. The frames are k = 12 t from -3 on: the
//! camera follows our car a quarter of a second (3 frames) behind, so one run serves both. Deck: how many frames.
//! Punches for each frame the table's index (0-250) and 1 if the sine is negative, else 0; the host fetches the
//! entries.
//!
//! The phase turns 1.7 / 12 radians a frame, which is 22.547 of the table's 1000 steps a turn: it is carried as whole
//! steps (0-999) and thousandths of a step, and rounded to the nearest step.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let frames = inp();
    // k = -3: 3 times 22.547 steps back from 0 is 932.359 steps on
    let (mut steps, mut thousandths) = (932, 359);
    for _ in 0..frames as usize {
        // the nearest step
        let mut at = steps;
        if thousandths >= 500 { if at == 999 { at = 0; } else { at += 1; } }
        // the second half of the turn is the first below zero; within a half, the table runs up to a quarter turn and
        // back down
        let (mut index, mut negative) = (at, 0);
        if index >= 500 { index -= 500; negative = 1; }
        if index > 250 { index = 500 - index; }
        out(index);
        out(negative);
        // 22.547 steps on (1000 is past a card: 977 and 978 are 1000 less 23 and 22)
        let (on, round) = if thousandths >= 453 { thousandths -= 453; (23, 977) } else { thousandths += 547; (22, 978) };
        if steps >= round { steps -= round; } else { steps += on; }
    }
}
