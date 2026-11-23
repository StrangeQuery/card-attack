//! Add, per frame of the dive for the jellyfish (places from their middles, facing). Deck: whether to take the
//! second away (1, else add), how many, then pairs. Punches, per pair, their sum (or difference).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let take = inp();
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        if take == 1 { out(a - b); } else { out(a + b); }
    }
}
