//! Shadow, once a frame of a UI layer, for all its shadows: where each one's soft edges are. Deck: how many; then
//! each: x and its group's move, y and its group's move, width, height, blur. Punches for each: where its left edge
//! starts to darken, where it is fully dark, where its right edge starts to lighten and where it ends; then the same
//! down the rows.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let x = inp() + inp();
        let y = inp() + inp();
        let width = inp();
        let height = inp();
        let blur = inp();
        edges(x, width, blur);
        edges(y, height, blur);
    }
}

/// The four edges of a span from `start`, `size` long, blurred `blur` either side.
fn edges(start: i16, size: i16, blur: i16) {
    let end = start + size;
    out(start - blur);
    out(start + blur);
    out(end - blur);
    out(end + blur);
}
