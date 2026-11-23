//! Doubles, for the deep's kelp blades (the host doubled them): each card twice over. Deck: how many, then the
//! cards. Punches each card added to itself.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let n = inp();
    for _ in 0..n {
        let x = inp();
        out(x + x);
    }
}
