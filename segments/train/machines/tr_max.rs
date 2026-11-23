//! Comparisons, for the train: of two longs (thousands, then the rest 0..999), the greater. Deck: the count, then pairs a, b
//! (longs). Punches each.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a_hi = inp();
        let a_lo = inp();
        let b_hi = inp();
        let b_lo = inp();
        // by the thousands, or by the rests when the thousands are equal
        let d = a_hi - b_hi;
        if d < 0 || (d == 0 && a_lo < b_lo) { out(b_hi); out(b_lo); } else { out(a_hi); out(a_lo); }
    }
}
