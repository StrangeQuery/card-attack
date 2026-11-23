//! Sines, for the swarm: between two entries of the printed table by tenths, and the sign. Deck: the count; then for
//! each angle, for its sine and then its cosine: the two entries, how far between (tenths) and 1 for the second half
//! turn (as the angle machine punches them, the entries looked up). Punches each angle's sine, cosine and their
//! negatives (thousandths, 999 for one).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut both = [0; 2];
        for i in 0..2 {
            let at = inp();
            let next = inp();
            let part = inp();
            let below = inp();
            // SAFETY: the table rises over its quarter turn by at most 7 a step, and part is a tenth 0..9
            unsafe { assume(part >= 0 && part <= 9); }
            let mut v = at + demokit::mul::times(next - at, part) / 10;
            if below > 0 { v = -v; }
            both[i] = v;
        }
        out(both[0]); out(both[1]); out(-both[0]); out(-both[1]);
    }
}
