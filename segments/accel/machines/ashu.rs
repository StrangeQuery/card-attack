//! Shadow axes, twice per frame of the accelerated drive while our car's shadow shows: how far out each pixel from its
//! middle is. Deck: its size r, the step (16ths of a pixel over r's units: 128 across, r in eighths; 256 down, r in
//! 16ths), how many pixels at most. Punches for each pixel k out u u, u = step k / r rounded down, and stops after
//! the first u of 18 or more (outside the shadow).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let r = inp();
    let step = inp();
    let n = inp();
    let mut u = 0;
    let mut rest = 0;
    for k in 0..n {
        if k > 0 {
            rest += step;
            while rest >= r { rest -= r; u += 1; }
        }
        let mut sq = 0;
        for _ in 0..u { sq += u; }
        out(sq);
        if u >= 18 { break; }
    }
}
