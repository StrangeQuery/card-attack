//! Strip, once per steep leaf per frame of the dive. Deck: the leaf's start down (tenths of a pixel from the
//! middle), how far it runs across and down (down not 0), and the first and last row to draw (counted from the
//! middle's, down). Punches, per row, how far across from the start its middle line is there: across (row's middle
//! - start) / down, rounded down. The line is walked a tenth at a time, the division's remainder carried, so that
//! no product passes a card.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let top = inp();
    let mut across = inp();
    let mut down = inp();
    let first = inp();
    let last = inp();
    if down < 0 { down = -down; across = -across; }
    // q + rem / down = across t / down, t tenths down from the start, rem 0..down - 1
    let mut q = 0;
    let mut rem = 0;
    let mut t = 0;
    for row in first..last + 1 {
        let goal = row * 10 + 5 - top;
        while t != goal {
            if t < goal { t += 1; rem += across; } else { t -= 1; rem -= across; }
            while rem >= down { rem -= down; q += 1; }
            while rem < 0 { rem += down; q -= 1; }
        }
        out(q);
    }
}
