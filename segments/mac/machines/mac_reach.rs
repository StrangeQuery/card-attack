//! Reach, once per row of the one-bit tunnel's blocks (setup): each block's distance from the middle in half pixels,
//! the longer offset times the secant, longer (1 + c / 200). Deck: for each of the row's 128 blocks, c = 200 (sec - 1)
//! from the slope table at its step (0-82), and its longer offset. Punches each distance, the extra rounded down
//! (c longer / 100, rounded down, halved).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");
use demokit::mul::by100;

pub fn main() {
    for _ in 0..128 {
        let c = inp();
        let longer = inp();
        out(longer + by100(c, longer) / 2);
    }
}
