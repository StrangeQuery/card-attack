//! The jet's lamps, once per frame of the accelerated drive. Deck: the frame f. Punches 1 if the jet is in the strip
//! of sky (frames 104 to 139), else 0; whether its wingtips are lit (f mod 4 below 2); whether its strobe is (f mod 6
//! is 0).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let f = inp();
    out(if f >= 104 && f < 140 { 1 } else { 0 });
    let mut four = f;
    while four >= 4 { four -= 4; }
    out(if four < 2 { 1 } else { 0 });
    let mut six = f;
    while six >= 6 { six -= 6; }
    out(if six == 0 { 1 } else { 0 });
}
