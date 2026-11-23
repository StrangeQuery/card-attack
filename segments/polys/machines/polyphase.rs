//! Phase, once per size of the polys' mesh. Deck: the count, then each corner's point on the sphere (x, y, z, in
//! hundredths). Punches the three waves' phases there, in 960ths of a turn: 5x + 3y, 6z and 9y (about 3.2x + 1.9y,
//! 3.9z and 5.8y radians), each brought within half a turn either way.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

#[inline(never)]
fn phase(w: i16) {
    let mut w = w;
    if w >= 480 { w -= 960; }
    if w < -480 { w += 960; }
    out(w);
}

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let x = inp();
        let y = inp();
        let z = inp();
        phase(5 * x + 3 * y);
        phase(6 * z);
        phase(9 * y);
    }
}
