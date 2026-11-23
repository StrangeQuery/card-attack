//! Shafts, once per screen row per frame of the dive. Deck: the row (0..95), then the two god-ray waves' phases this
//! frame (each a whole step 0..59 of a 60-step turn and hundredths of a step). Across the screen the waves run along
//! s = column + 0.45 row, 0.86 and 0.41 steps a column (0.39 and 0.18 a row). Punches, per pixel of the row's 192,
//! each wave's step (0..59).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let row = inp();
    let mut first = inp();
    let mut first_part = inp();
    let mut second = inp();
    let mut second_part = inp();
    // the row's start: 0.39 and 0.18 of a step per row down
    for _ in 0..row {
        first_part += 39;
        second_part += 18;
        if first_part >= 100 { first_part -= 100; first += 1; }
        if second_part >= 100 { second_part -= 100; second += 1; }
    }
    for _ in 0..192 {
        if first >= 60 { first -= 60; }
        if second >= 60 { second -= 60; }
        out(first);
        out(second);
        first_part += 86;
        second_part += 41;
        if first_part >= 100 { first_part -= 100; first += 1; }
        if second_part >= 100 { second_part -= 100; second += 1; }
    }
}
