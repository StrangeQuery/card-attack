//! Tone, per frame of the polys: the level and highlight of each pixel the blobs cover, for the dot machine to make
//! its colour (the level's parts: 0.75, 0.86, 0.86 for the first blob's grey-blue, 0.86, 0.6, 0.45 for the second's
//! copper, with these added). Deck: the count, then for each pixel its light (in tenths of a step) and highlight (in
//! thousandths). Punches the level, 40 + the light, at most 250 (two-hundredths, as it comes: no bands), and the
//! highlight to add to red and blue (0.04 of it) and to green (0.03), each a tenth at a time.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut level = 40 + inp();
        if level > 250 { level = 250; }
        let tenths = inp() / 10;
        out(level);
        out(4 * tenths / 10);
        out(3 * tenths / 10);
    }
}
