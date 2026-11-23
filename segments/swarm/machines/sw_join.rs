//! The quotient, for the swarm's long division: its digits made a long. Deck: the count; then each: how many digits
//! and the digits, the largest first. Punches each as a long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let digits = inp();
        let (mut qh, mut ql) = (0, 0);
        for _ in 0..digits {
            let top = digit(ql, 0);
            qh = qh * 10 + top;
            ql = (ql - top * 100) * 10 + inp();
        }
        out(qh); out(ql);
    }
}
