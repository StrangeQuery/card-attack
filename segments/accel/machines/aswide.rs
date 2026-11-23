//! Street pixel widths, once per row of the street per frame of the accelerated drive, for the pixels near a line's
//! edge: how wide a pixel is there (640ths of a unit across, at least 0.06 units). Deck: how many, then each its
//! distance (quarters). Punches each 48 a / 110, at least 38.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let far = inp();
        let mut w = 0;
        let mut sum = -110;
        for _ in 0..48 {
            sum += far;
            while sum >= 0 { sum -= 110; w += 1; }
        }
        out(if w < 38 { 38 } else { w });
    }
}
