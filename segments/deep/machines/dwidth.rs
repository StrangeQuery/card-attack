//! Width, once per leaf per frame of the dive (kelp, seagrass). Deck: the leaf's half width w (tenths), whether it
//! is a blade (1: it tapers to its ends, sin(pi u)^0.7 from a table, else as wide all along), how many pixels, then
//! for each how far along it is (hundredths of its length). Punches, per pixel, the leaf's half width there
//! (tenths): w t / 100 by w's digits.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let taper = tables::LEAF_TAPER;
    let width = inp();
    let blade = inp();
    let count = inp();
    for _ in 0..count {
        let along = inp() / 5;
        // SAFETY: the host deals 0..100
        unsafe { assume(along >= 0 && along <= 20) };
        let mut part = width;
        if blade == 1 {
            let t = taper[along as usize];
            part = 0;
            for k in 0..3 {
                for _ in 0..digit(width, 2 - k) { part += t; }
                if k < 2 { part /= 10; }
            }
        }
        out(part);
    }
}
