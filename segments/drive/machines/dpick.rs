//! Pick, per frame of the drive. Deck: where to start and how much further, the step, how many, then a table.
//! Punches that many of the table's cards, from the start, a step apart: a stretch of the street's plan under the
//! camera, the blocks ahead, the street's middle at each block, a wall's column of the window texture, a row of a
//! car's picture.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let start = inp() + inp();
    let step = inp();
    let count = inp();
    for _ in 0..start { inp(); }
    for _ in 0..count {
        out(inp());
        for _ in 1..step { inp(); }
    }
}
