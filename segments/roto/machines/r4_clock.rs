//! The heart rotozoomer's clock, once a tile of a frame (it was the host's). Deck: the frame f. Punches the
//! two angles it turns by, in 48ths: f mod 48, and 2 f / 3 mod 48 (two thirds as fast).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let f = inp();
    let mut a = f;
    while a >= 48 { a -= 48; }
    let mut x = f + f;
    let mut b = 0;
    while x >= 3 { x -= 3; b += 1; }
    while b >= 48 { b -= 48; }
    out(a);
    out(b);
}
