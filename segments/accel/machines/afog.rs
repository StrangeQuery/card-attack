//! Haze, as many as the host has, per frame of the accelerated drive. Deck: how many, then each a distance (quarters).
//! Punches each its haze, 32nds: none to 56 units, all by 150: (a - 224) 4 / 47 rounded down, 0-32.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let mut haze = 0;
        let mut sum = 0;
        if a > 224 { for _ in 0..4 { sum += a - 224; while sum >= 47 { sum -= 47; haze += 1; } } }
        out(if haze > 32 { 32 } else { haze });
    }
}
