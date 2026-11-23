//! The ray's ball's hop (its height) and the floor's scroll, once a frame. Deck: the frame f (0..119). Punches the
//! height, 14 + 10 (1 - (2u - 1)^2) rounded half up for u = (f mod 24) / 24, which is 14 + 5 (144 - m^2) / 72 for
//! m = f mod 24 - 12 (a bounce every two seconds); then the floor's scroll, 3 f mod 20 and 2 f mod 20.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// x mod m, by taking m away (x 0 or more).
fn modulo(mut x: i16, m: i16) -> i16 {
    while x >= m { x -= m; }
    x
}

pub fn main() {
    let f = inp();
    let m = modulo(f, 24) - 12;
    let p = (144 - m * m) * 5;
    let mut h = 0;
    let mut r = p + 36;
    while r >= 72 { r -= 72; h += 1; }
    out(14 + h);
    out(modulo(3 * f, 20));
    out(modulo(2 * f, 20));
}
