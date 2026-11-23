//! Orca distance, for the orcas in the dive at 1536x768: how far ahead each of the mesh kit's depths is, in whole
//! units, for one orca this frame. Deck: the distance at depth 0 (whole units) and the rest (0..859, of 860ths),
//! the orca's length in units, how many depths. Punches, for each depth d from 0, fl((zero + d) length / 860): the
//! length added a depth at a time, the 860s carried out as they fill.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut ahead = inp();
    let mut rest = inp();
    let length = inp();
    for _ in 0..inp() {
        out(ahead);
        rest += length;
        while rest >= 860 { rest -= 860; ahead += 1; }
    }
}
