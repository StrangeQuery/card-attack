//! Order, once per screen row per frame of the dive while the portal shows. Deck: how many pixels, then for each
//! where its ray crosses the portal's plane, across and up from the middle (tenths). Punches, per pixel, the larger
//! and the smaller of the two parts' sizes, then which eighth of the turn the point is in: 1 if up is the larger
//! part (else 0), 1 if across is below zero, 1 if up is.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let across = inp();
        let up = inp();
        let mut large = abs(across);
        let mut small = abs(up);
        let mut steep = 0;
        if small > large { large = small; small = abs(across); steep = 1; }
        out(large);
        out(small);
        out(steep);
        out(if across < 0 { 1 } else { 0 });
        out(if up < 0 { 1 } else { 0 });
    }
}
