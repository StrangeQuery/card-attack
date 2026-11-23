//! Ghost rows, per row of a ghost per frame of the accelerated drive. Deck: the row's v v, how many pixels, then each
//! its u u (aghu's). Punches each u u + v v, at most 576 (none of the ghost's profile).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let v = inp();
    for _ in 0..inp() {
        let u = inp();
        out(if u > 576 - v { 576 } else { u + v });
    }
}
