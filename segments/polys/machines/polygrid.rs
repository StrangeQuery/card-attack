//! Grid, once per size of the polys' mesh. Deck: the bands of corners (lat) and the corners around (lon). Punches
//! the bands' angles down from the top, 480 (2 i + 1) / (2 lat) for i = 0..lat, then the angles around, 960 j / lon
//! for j = 0..lon, in 960ths of a turn (each cut toward zero, and brought within half a turn either way). Each list steps by 960 / its divisor, kept as a
//! whole and a remainder: 480 divided once, then added twice a step.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// Punch count values: start (a whole and a remainder by divisor), then stepping by twice 480 / divisor.
#[inline(never)]
fn steps(divisor: i16, count: i16, start_whole: i16, start_rest: i16) {
    let (whole, rest) = divide(480, divisor);
    let (mut at, mut over) = (start_whole, start_rest);
    for _ in 0..count {
        out(if at >= 480 { at - 960 } else { at });     // within half a turn either way
        for _twice in 0..2 {
            at += whole;
            over += rest;
            if over >= divisor { over -= divisor; at += 1; }
        }
    }
}

pub fn main() {
    let bands = inp();
    let around = inp();
    let (whole, rest) = divide(480, 2 * bands);
    steps(2 * bands, bands, whole, rest);
    steps(around, around, 0, 0);
}
