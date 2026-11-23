//! Street pixel lengths, once per row of the street per frame of the accelerated drive, for the pixels on the stripe:
//! how long a pixel is along the street there (64ths of a unit, at least 0.12 units). Deck: how many, then each its
//! distance (quarters). Punches each (a / 8) (a / 8) / 3, 8 to 999.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let eighth = inp() / 2 / 2 / 2;
        let mut w = 0;
        let mut sum = -3;
        for _ in 0..eighth {
            sum += eighth;
            while sum >= 0 { sum -= 3; if w < 999 { w += 1; } }
        }
        out(if w < 8 { 8 } else { w });
    }
}
