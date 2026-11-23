//! Levels, as many as the host has, per frame of the accelerated drive: a base and some times a number, kept to a
//! most. Deck: how many, then each: the base, how many times, the number (0 or more), the most. Punches each
//! min(most, base + times number).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut level = inp();
        let times = inp();
        let v = inp();
        let most = inp();
        for _ in 0..times { if level < most { level += v; } }
        out(if level > most { most } else { level });
    }
}
