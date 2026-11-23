//! The tone curve, for the train, a little more colour, the second half: each channel plus its twentieth (from the
//! host's table), held to 0..255. Deck: the count, then each pixel's three channels and three twentieths. Punches the
//! three.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let c = inp();
        for k in 0..3 {
            let t = if k == 0 { a } else if k == 1 { b } else { c };
            let mut v = t + inp();
            if v < 0 { v = 0; }
            if v > 255 { v = 255; }
            out(v);
        }
    }
}
