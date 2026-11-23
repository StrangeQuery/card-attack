//! Changes between samples, for the train: b - a, and for values that repeat every 1000 the shorter way round,
//! -500..499. Deck: the count, 1 when they repeat (else 0); then pairs a, b (cards). Punches each change.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let wrap = inp();
    for _ in 0..count {
        let a = inp();
        let mut d = inp() - a;
        if wrap > 0 {
            if d >= 500 { d -= 999; d -= 1; }
            if d < -500 { d += 999; d += 1; }
        }
        out(d);
    }
}
