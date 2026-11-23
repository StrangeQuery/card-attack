//! The camera's path across the street, or our car's lane, a frame at a time, in eighths of a unit from the street's
//! middle: 2.5 units (20 eighths), less 4 units times the swerve past the slow car ahead, plus the wobble, rounded to
//! the nearest eighth. The frames are k = 12 t from -3 on, as aphase's and aswerve's: the camera's path is the car's
//! lane 3 frames earlier, so one run serves both. Deck: how many frames; then for each frame, as the host fetched
//! them: the wobble (3.2 sin, in 216ths of an eighth, from data/lanesine.txt at aphase's index), 1 if the wobble is
//! negative (else 0), and the swerve as whole eighths and 216ths over (from data/laneswerve.txt at aswerve's
//! entry). Punches each frame's place.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let frames = inp();
    for _ in 0..frames as usize {
        let mut wobble = inp();
        if inp() == 1 { wobble = 0 - wobble; }
        let whole = inp();
        let over = inp();
        // the 216ths add up, with a half (108) for the rounding, and whole eighths are carried out of them
        let mut lane = 20 - whole;
        let mut part = wobble - over + 108;
        while part >= 216 { part -= 216; lane += 1; }
        while part < 0 { part += 216; lane -= 1; }
        out(lane);
    }
}
