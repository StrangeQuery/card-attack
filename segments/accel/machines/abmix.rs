//! Bloom's amount, per 4 x 4 block that has one per channel per frame of the accelerated drive. Deck: how many, then
//! each: the blur of 7 and of 19 (steps). Punches each (495 the first + 330 the second) / 100, rounded down (tenths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut q = 0;
        let mut rest = 0;
        // 495 = 4 hundreds and 95; 330 = 3 hundreds and 30
        for _ in 0..inp() { q += 4; rest += 95; if rest >= 100 { rest -= 100; q += 1; } }
        for _ in 0..inp() { q += 3; rest += 30; if rest >= 100 { rest -= 100; q += 1; } }
        out(q);
    }
}
