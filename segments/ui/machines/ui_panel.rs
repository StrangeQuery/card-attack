//! Panel, once a frame of a UI layer, for all its boxes: where each row of each box starts and ends, its corners
//! rounded. Deck: how many boxes; then each: x and its group's move, y and its group's move, width, height, radius;
//! then for each row, the corner machine's whole pixels for it and how many partly covered ones it punched (the
//! radius and 0 for a row with no corner). Punches for each box its top row's y; then for each row where its partly
//! covered pixels start, its whole ones, where the whole ones end and where the partly covered ones do.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let left = inp() + inp();
        out(inp() + inp());
        let right = left + inp();
        let height = inp();
        let radius = inp();
        let inner_left = left + radius;
        let inner_right = right - radius;
        for _ in 0..height {
            let whole = inp();
            let end = whole + inp();
            out(inner_left - end);
            out(inner_left - whole);
            out(inner_right + whole);
            out(inner_right + end);
        }
    }
}
