//! Place, once a frame of the Mac's desk, for every place on it: a corner of a window (or of the screen, all 0) and a
//! distance from it, added, as the Mac's own resources gave a window's parts in the window's coordinates. Deck: how
//! many windows; then each: its frame's left, top, right and bottom (left + right at most 999), how many places, and
//! each place: which of the window's numbers it is from (0 left, 1 top, 2 right, 3 bottom, 4 the middle across,
//! (left + right) / 2) and how far. Punches each place.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let windows = inp();
    for _ in 0..windows {
        let left = inp();
        let top = inp();
        let right = inp();
        let bottom = inp();
        let middle = (left + right) / 2;
        let places = inp();
        for _ in 0..places {
            let which = inp();
            let distance = inp();
            let corner = match which {
                0 => left,
                1 => top,
                2 => right,
                3 => bottom,
                _ => middle,
            };
            out(corner + distance);
        }
    }
}
