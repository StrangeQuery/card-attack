//! Push, twice per frame of the degauss's jolt: how far each row of the frame is pushed across, or each column
//! down, for each colour. Deck: the colours' spread (tenths of a pixel), how many lines; then each line's push
//! (tenths). Punches per line the first colour's push (the line's and the spread), the second's (the line's) and
//! the third's (the line's less the spread), in whole pixels, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// v / 10 rounded down.
#[inline(never)]
fn pixels(v: i16) -> i16 {
    let q = v / 10;
    if v < q * 10 { q - 1 } else { q }
}

pub fn main() {
    let spread = inp();
    let lines = inp();
    for _ in 0..lines {
        let push = inp();
        out(pixels(push + spread));
        out(pixels(push));
        out(pixels(push - spread));
    }
}
