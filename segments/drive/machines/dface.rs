//! The blocks' faces, once per frame of the drive. Deck: the camera across (eighths), the street's middle at each of
//! the nine blocks from the camera's on, then for each block the left building's setback from it, its height (in
//! bands), tone and window texture place, and the right one's. Punches for each block and side: the face across
//! from the camera in eighths and in quarters (rounded down), the height, the tone and the texture place.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let camera = inp();
    let mut middles = [0; 9];
    for i in 0..9 { middles[i] = inp() - camera; }
    for block in 0..9 {
        for side in 0..2 {
            let setback = inp();
            let eighths = if side == 0 { middles[block] - setback } else { middles[block] + setback };
            out(eighths);
            // half of it rounded down, for a negative one too
            out(if eighths < 0 { -((1 - eighths) / 2) } else { eighths / 2 });
            out(inp());
            out(inp());
            out(inp());
        }
    }
}
