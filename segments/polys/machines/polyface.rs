//! Face, once per frame of the polys: each covered pixel's normal turned to face us. Deck: the count, then each
//! pixel's normal (three parts, in thousandths). Punches the normal, its depth part turned round if it is above 0
//! (facing away: carried across a triangle at the blob's edge, it can tip a little past the edge; turned round
//! whole, its other parts would point the other way and it would take the light from the other side), then a tenth
//! of each part.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let x = inp();
        let y = inp();
        let mut z = inp();
        if z > 0 { z = -z; }
        out(x);
        out(y);
        out(z);
        out(x / 10);
        out(y / 10);
        out(z / 10);
    }
}
