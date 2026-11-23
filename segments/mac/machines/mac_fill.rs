//! Fill, for each frame of the desk: boxes of one colour, the desk's filled rectangles, window outlines' sides, text
//! boxes' backs and the flag's frame. Deck: how many boxes, then each: its colour (1 black, 0 white), its first and
//! last columns and its first and last rows (kept to the screen). Punches each box's pixels' colour, row by row.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let colour = inp();
        let left = inp();
        let right = inp();
        let top = inp();
        let bottom = inp();
        let mut y = top;
        while y <= bottom {
            let mut x = left;
            while x <= right {
                out(colour);
                x += 1;
            }
            y += 1;
        }
    }
}
