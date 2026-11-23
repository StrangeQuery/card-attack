//! Ghost axes, per ghost per frame of the accelerated drive. Deck: its radius rr (16ths), how many pixels. Punches for
//! each pixel k out from its middle u u, u = 256 k / rr rounded down, at most 24.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let rr = inp();
    let n = inp();
    let mut u = 0;
    let mut rest = 0;
    for k in 0..n {
        if k > 0 && u < 24 {
            rest += 256;
            while rest >= rr && u < 24 { rest -= rr; u += 1; }
        }
        let mut sq = 0;
        for _ in 0..u { sq += u; }
        out(sq);
    }
}
