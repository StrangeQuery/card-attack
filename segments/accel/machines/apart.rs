//! Parts, as many as the host has, per frame of the accelerated drive: a base and a number times m over d. Deck: how
//! many, then each: the base, v (0 or more), m and d (m below d + 1000 less the most v m / d can reach). Punches
//! each base + v m / d, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut q = inp();
        let v = inp();
        let m = inp();
        let d = inp();
        let mut sum = 0;
        for _ in 0..v {
            sum += m;
            while sum >= d { sum -= d; q += 1; }
        }
        out(q);
    }
}
