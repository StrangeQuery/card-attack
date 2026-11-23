//! Bounds, once per side per frame of the dive for the kelp's and seagrass's leaves (across, then down). Deck: how
//! many leaves, then for each the start and end along that side (tenths of a pixel from the middle) and its half
//! width (tenths). Punches, per leaf, the first and last column (or row, down) a pixel of it can be in, counted from
//! the middle's: (lesser - width - 10) / 10 and (greater + width + 19) / 10 rounded down, each split into tens and
//! units 0..9 first so that nothing passes a card.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let width = inp();
        let mut low = a;
        let mut high = b;
        if b < a { low = b; high = a; }
        let mut tens = low / 10;
        let mut units = low - tens * 10 - width - 10;
        while units < 0 { units += 10; tens -= 1; }
        out(tens);
        tens = high / 10;
        units = high - tens * 10 + width + 19;
        while units < 0 { units += 10; tens -= 1; }
        while units >= 10 { units -= 10; tens += 1; }
        out(tens);
    }
}
