//! Index, per frame of the dive for the fog on the kelp and the fish. Deck: how many, then distances ahead (tenths).
//! Punches, per distance, its whole units, at most 49: the fog table's place.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut units = inp() / 10;
        if units > 49 { units = 49; }
        out(units);
    }
}
