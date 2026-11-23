//! Order, for the swarm's angles: two sizes (longs), the smaller first. Deck: the count, then pairs a, b. Punches
//! for each the smaller and the larger, then 1 if a was the larger (else 0), and 1 if both are 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let (ah, al) = (inp(), inp());
        let (bh, bl) = (inp(), inp());
        if ah > bh || (ah == bh && al > bl) {
            out(bh); out(bl); out(ah); out(al); out(1); out(0);
        } else {
            out(ah); out(al); out(bh); out(bl); out(0);
            if bh == 0 && bl == 0 { out(1); } else { out(0); }
        }
    }
}
