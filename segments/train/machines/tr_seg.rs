//! Values between samples, for the train: across a run of a surface's pixels, a value worked out at two samples is
//! carried between them, the step a pixel in thousandths, the value its whole part (and, for a value that repeats,
//! md 1000). Deck: the count, 1 when the values repeat (else 0); then each segment: how many pixels, the first sample's
//! value, and the step (a long, thousandths). Punches each pixel's value.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let wrap = inp();
    for _ in 0..count {
        let n = inp();
        let mut v = inp();
        let s_hi = inp();
        let s_lo = inp();
        let mut f = 0;
        for _ in 0..n {
            out(v);
            v += s_hi;
            // the thousandths: past 999, one more
            let over = f - (999 - s_lo);
            if over > 0 { f = over - 1; v += 1; } else { f = over + 999; }
            if wrap > 0 {
                if v > 999 { v -= 999; v -= 1; }
                if v < 0 { v += 999; v += 1; }
            }
        }
    }
}
