//! Motion as table places, for the train's blur: a block's flow from the camera's travel and from its turn, across
//! and down (u at most 400, r at most 599 in size), each sum held to -400..400, plus 400. Deck: the count, then items:
//! ux, rx, uy, ry. Punches the two places.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        for _ in 0..2 {
            let mut s = inp();
            s += inp();
            if s > 400 { s = 400; }
            if s < -400 { s = -400; }
            out(s + 400);
        }
    }
}
