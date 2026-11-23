//! Cell, once per frame of the polys: where each pixel of the blobs falls on the shadow map. Deck: the count, then
//! for each pixel its column and row, and the map's move less 0.80 and 0.97 of its depth (from the dot machine), in
//! quarter pixels. Its middle (4 x + 2, 4 y + 2) moved so lands on the map (768 x 384 quarter pixels, 192 x 96
//! places). Punches the place's column and row, or -1 and -1 if it is off the map.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// A place on the map from quarter pixels (0 or more): divided by 4, by subtracting, ten places at a time, then one.
fn place(at: i16) -> i16 {
    let mut rest = at;
    let mut n = 0;
    while rest >= 40 { rest -= 40; n += 10; }
    while rest >= 4 { rest -= 4; n += 1; }
    n
}

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let x = inp();
        let y = inp();
        let across = 4 * x + 2 + inp();
        let down = 4 * y + 2 + inp();
        if across < 0 || across > 767 || down < 0 || down > 383 {
            out(-1);
            out(-1);
        } else {
            out(place(across));
            out(place(down));
        }
    }
}
