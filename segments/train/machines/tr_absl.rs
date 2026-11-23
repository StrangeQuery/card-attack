//! Within a width, for the train's ground: whether a place across (a long) is nearer the middle than h (a long), and
//! whether it is below 0. Deck: the count, then items: x, h (longs). Punches the two flags.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut xh = inp();
        let mut xl = inp();
        let hh = inp();
        let hl = inp();
        let neg = if xh < 0 { 1 } else { 0 };
        // the size: -x is (-xh - 1, 1000 - xl), or (-xh, 0) where xl is 0
        if neg == 1 { if xl == 0 { xh = -xh; } else { xh = -xh - 1; xl = 999 - xl + 1; } }
        out(if xh < hh || (xh == hh && xl < hl) { 1 } else { 0 });
        out(neg);
    }
}
