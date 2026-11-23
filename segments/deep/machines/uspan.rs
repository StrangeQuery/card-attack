//! Span, twice per point of a jellyfish's bell per frame of the dive (across, then down). Deck: whether down (1) or
//! across (0), how many points, then for each its place on the screen from the middle (tenths of a pixel, across
//! and up) and its radius (tenths of a pixel). Punches, per point, its radius kept within 5..31 and the first and
//! last column (or row, down) of pixels it might cover, counted from the middle's pixel: (place - radius) / 10 and
//! (place + radius + 9) / 10 rounded down, the place split into tens and units first so that nothing passes a card.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let down = inp();
    let count = inp();
    for _ in 0..count {
        let mut v = inp();
        let up = inp();
        if down == 1 { v = -up; }
        let mut radius = inp();
        if radius < 5 { radius = 5; }
        if radius > 31 { radius = 31; }
        out(radius);
        let mut tens = v / 10;
        let mut units = v - tens * 10;
        if units < 0 { units += 10; tens -= 1; }
        // the tens carried down for units - radius and up for units + radius + 9
        let mut first = tens;
        let mut below = units - radius;
        while below < 0 { below += 10; first -= 1; }
        let mut last = tens;
        let mut above = units + radius + 9;
        while above >= 10 { above -= 10; last += 1; }
        out(first);
        out(last);
    }
}
