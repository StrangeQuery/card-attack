//! Nebula, once per row of the polys' sky (it doesn't move). Deck: the row (y) and 99 sin 14 y. Punches the three
//! waves' angles at each pixel of the row, in 960ths of a turn (0..959): 9 x + 1.5 (99 sin 14 y), 17 y + 7 x and 12 y
//! + 12 x, stepped across the row.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// Bring an angle back into a turn, from below 1920.
#[inline(never)]
fn turn(a: i16) -> i16 { if a >= 960 { a - 960 } else { a } }

pub fn main() {
    let y = inp();
    let bend = inp();
    let mut first = bend + 5 * bend / 10;
    if first < 0 { first += 960; }
    let mut second = 0;
    let mut third = 0;
    for _ in 0..y {
        second = turn(second + 17);
        third = turn(third + 12);
    }
    for _x in 0..192 {
        out(first);
        out(second);
        out(third);
        first = turn(first + 9);
        second = turn(second + 7);
        third = turn(third + 12);
    }
}
