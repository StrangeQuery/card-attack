//! Signs, for the swarm's arithmetic: whether a number of three cards (millions, thousands, rest) is below 0, and
//! whether it is 0. Deck: the count, then the numbers. Punches for each 1 if it is below 0 (else 0), then 1 if it is
//! 0 (else 0).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let h2 = inp();
        let h1 = inp();
        let h0 = inp();
        if h2 < 0 { out(1); } else { out(0); }
        if h2 == 0 && h1 == 0 && h0 == 0 { out(1); } else { out(0); }
    }
}
