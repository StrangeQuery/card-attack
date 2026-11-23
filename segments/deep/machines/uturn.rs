//! Turn, once per screen row per frame of the dive while the portal shows. Deck: how many, then for each an angle
//! within an eighth of a turn (tenths of a step of a 60-step turn, 0..75) and which eighth (three flags, as the
//! order machine punches them: up the larger part, across below zero, up below zero). Punches, per angle, it
//! turned to its eighth: 0..599 tenths of a step, counted from across toward up.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut angle = inp();
        if inp() == 1 { angle = 150 - angle; }
        if inp() == 1 { angle = 300 - angle; }
        if inp() == 1 && angle > 0 { angle = 600 - angle; }
        out(angle);
    }
}
