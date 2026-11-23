//! Sine, four times per frame of the Rubik's cube (the yaw, the pitch, the roll, and the turning layer's turn).
//! Deck: the angle in tenths of a degree, as its quarter turn (0-3) and the rest (0-899); then twice the printed
//! table 999 sin(k degrees), k = 0..91, read one card at a time. Between whole degrees the table is read along a
//! straight line: this + (next - this) tenths / 10. The first table gives the sine of the rest, the second its
//! cosine (the sine of 90 degrees less it). Punches 999 sin and 999 cos of the angle.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let quarter = inp();
    let rest = inp();
    let mut at = rest;
    let mut sin = 0;
    let mut cos = 0;
    for _ in 0..2 {                     // the sine, then the cosine
        // the degree and the tenths past it, by repeated subtraction
        let mut tenths = at;
        let mut degree = 0;
        while tenths >= 10 { tenths -= 10; degree += 1; }
        for _ in 0..degree { inp(); }   // skip to the degree
        let low = inp();
        let high = inp();
        for _ in degree..90 { inp(); }  // and past the rest of the table
        // (high - low) tenths / 10: the product by adding, then the tenth by subtracting
        let mut rise = 0;
        for _ in 0..tenths { rise += high - low; }
        let mut value = low;
        while rise >= 10 { rise -= 10; value += 1; }
        sin = cos;
        cos = value;
        at = 900 - rest;
    }
    // a quarter turn on: (sin, cos) becomes (cos, -sin)
    for _ in 0..quarter {
        let t = sin;
        sin = cos;
        cos = -t;
    }
    out(sin);
    out(cos);
}
