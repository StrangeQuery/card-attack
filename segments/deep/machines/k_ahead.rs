//! Ahead, for the dive's ending at 1536x768: how far ahead each of the mesh kit's depths is, in whole units, for one
//! model this frame. Deck: the distance at depth 0 (whole units) and the rest (of S), S (the model's units to the
//! dive's unit), how many depths. Punches, for each depth d from 0, fl((zero + d) / S): a depth at a time, the S
//! carried out as they fill.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut ahead = inp();
    let mut rest = inp();
    let s = inp();
    for _ in 0..inp() {
        out(ahead);
        rest += 1;
        if rest >= s { rest -= s; ahead += 1; }
    }
}
