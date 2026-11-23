//! Block mean, once per row of blocks per channel per frame of an un-pixelate step in the cut. Deck: how many
//! blocks, how many values a block has (4, 16 or 64: 2 x 2, 4 x 4 or 8 x 8 pixels), then each block's values
//! (0-255). Punches each block's mean, rounded half up: the sum kept as whole means and a remainder, so it never
//! passes a card.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let blocks = inp();
    let size = inp();
    let eight = 8 * size;
    for _ in 0..blocks {
        let mut mean = 0;
        let mut left = 0;
        for _ in 0..size {
            left += inp();
            // division by repeated subtraction: eight means at a time while it can, then one
            while left >= eight {
                left -= eight;
                mean += 8;
            }
            while left >= size {
                left -= size;
                mean += 1;
            }
        }
        if left + left >= size {
            mean += 1;
        }
        out(mean);
    }
}
