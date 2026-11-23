//! Sample, a few times per pixel per frame of the dive where the seabed is hit. Deck: x as a whole part and
//! hundredths 0..99, then a row of the height map: 201 heights in tenths, x = -100..100. Punches the height under
//! x: the two heights either side, the step between them taken in tenths of a unit (off the map, the edge).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let x = inp();
    let x_part = inp();
    let mut index = x + 100;
    let mut part = x_part / 10;
    if index < 0 { index = 0; part = 0; }
    if index > 199 { index = 199; part = 9; }
    // the cards before x's are read and dropped, ten at a time while there are ten
    while index >= 10 {
        index -= 10;
        inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp();
    }
    for _ in 0..index { inp(); }
    let left = inp();
    let right = inp();
    let mut rise = 0;
    for _ in 0..part { rise += right - left; }
    out(left + rise / 10);
}
