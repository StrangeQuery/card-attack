//! Lamps, once per frame of the drive: where the street lamps are, from the camera. A lamp stands every block, 60
//! quarters in, 53 eighths (6.6 units) either side of the street's middle. Deck: how far the camera is into its
//! block (quarters), the camera across (eighths), then for the seven blocks from the camera's on the two entries of
//! the street's plan either side of the lamps (15 a block apart, 7 and 8 in). Punches for each lamp, the left one
//! then the right in each block: how far across from the camera (eighths) and along ahead of it (quarters).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut ahead = 60 - inp();
    let camera = inp();
    for _ in 0..7 {
        let before = inp();
        let mut half = inp() - before + 16;   // half way to the next entry, rounded down: lifted to stay above 0
        let mut middle = before - 8;
        while half >= 2 { half -= 2; middle += 1; }
        out(middle - 53 - camera);
        out(ahead);
        out(middle + 53 - camera);
        out(ahead);
        ahead += 120;
    }
}
