//! Sum, per frame of the dive for the kelp's points. Deck: how many, then triples. Punches, per triple, its sum.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        out(inp() + inp() + inp());
    }
}
