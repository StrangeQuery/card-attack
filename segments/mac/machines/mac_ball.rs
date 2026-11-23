//! Ball, once for the Wikipedia page's sphere: each row across the sphere, each pixel's light, -50 x - 60 y + 62 z in
//! 7000ths (z the whole square root of 70^2 - x^2 - y^2, the height of the sphere's face over the pixel). Across a row
//! the square under the root changes by 2x + 1 a pixel, so the root is kept as it goes, with what the square is over
//! its root's square (never more than twice the root): a step up or down when that passes either way. The light is
//! carried in hundreds and the rest (it passes a card). Deck: the count; each row: how many pixels, the first one's
//! x, the root there and what the square is over its square, the light there (hundreds, rest). Punches each pixel's
//! light (hundreds, rest).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut left = inp();
        let mut x = inp();
        let mut root = inp();
        let mut over = inp();
        let mut hundreds = inp();
        let mut rest = inp();
        loop {
            out(hundreds);
            out(rest);
            left -= 1;
            if left == 0 { break }
            // the next pixel: the square under the root less 2x + 1, the light 50 less
            over -= 2 * x + 1;
            x += 1;
            let mut change = -50;
            while over > 2 * root {
                over -= 2 * root + 1;
                root += 1;
                change += 62;
            }
            while over < 0 {
                root -= 1;
                over += 2 * root + 1;
                change -= 62;
            }
            rest += change;
            while rest >= 100 { rest -= 100; hundreds += 1; }
            while rest < 0 { rest += 100; hundreds -= 1; }
        }
    }
}
