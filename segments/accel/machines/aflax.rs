//! Flare axes, per light per term per frame of the accelerated drive: how far along a falloff each pixel out from a
//! light is. Deck: the first place a0 (16ths from the light), how many pixels, m and d. Punches for each pixel out (a
//! = a0, a0 + 16, ...) a m / d rounded down, at most 192, and stops after the first of 192 or more.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let a0 = inp();
    let n = inp();
    let m = inp();
    let d = inp();
    let mut q = 0;
    let mut rest = 0;
    let mut step = a0;
    for _ in 0..n {
        for _ in 0..step {
            rest += m;
            while rest >= d { rest -= d; q += 1; }
        }
        step = 16;
        if q >= 192 { out(192); break; }
        out(q);
    }
}
