//! Half, for the mesh kit's lift: how much a normal carried across a triangle falls short of a unit, halved. Deck:
//! the count, then each normal's parts squared in hundredths (from the squares' table). Punches for each half of what
//! their sum falls short of 98 (a unit's, 99 long), rounded toward zero (below 0 for a normal a little long).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        out((98 - inp() - inp() - inp()) / 2);
    }
}
