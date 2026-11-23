//! Wall hits, once per wall a column's ray meets per frame of the accelerated drive: the sizes the wall's machines
//! work with. Deck: how many, then each: the ray across, the face's distance across (eighths), the block's start
//! (quarters ahead), how far along an end wall the ray meets it (-1 for a face). Punches each |across|, |e|, the
//! start at least 16, and 1 for an end wall (0 a face).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        out(abs(inp()));
        out(abs(inp()));
        let start = inp();
        out(if start < 16 { 16 } else { start });
        out(if inp() >= 0 { 1 } else { 0 });
    }
}
