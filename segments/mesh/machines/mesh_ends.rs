//! Ends, for the mesh kit: a span's two crossings in order across its row. Deck: the count, then each span's two
//! pixels (where its triangle's two edges cross the row). Punches for each 1 if the second is left of the first (else
//! 0), then how many pixels the span covers (the right less the left).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        if b < a { out(1); out(a - b); } else { out(0); out(b - a); }
    }
}
