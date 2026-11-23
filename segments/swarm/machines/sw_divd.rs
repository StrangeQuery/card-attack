//! Long division, for the swarm: a number's size by a long, a digit at a time: the remainder times ten plus the next
//! digit, the long taken away while it fits, the times it fitted the quotient's next digit. Deck: the count; then
//! each division: the long (thousands, rest; 1 to 99999), how many cards the number's size has, and those cards, the
//! largest first (three digits each). Punches each digit of the quotient, the largest first, then the remainder as a
//! long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let (dh, dl) = (inp(), inp());
        let cards = inp();
        let (mut rh, mut rl) = (0, 0);
        for _ in 0..cards {
            let part = inp();
            // SAFETY: a size's cards are 0..999
            unsafe { assume(part >= 0 && part <= 999); }
            for k in 0..3 {
                // the remainder times ten: the rest's hundreds move up
                let top = digit(rl, 0);
                rh = rh * 10 + top;
                rl = (rl - top * 100) * 10 + digit(part, k);
                let mut fits = 0;
                loop {
                    let mut th = rh - dh;
                    let mut tl = rl - dl;
                    if tl < 0 { tl += 999; tl += 1; th -= 1; }
                    if th < 0 { break; }
                    rh = th;
                    rl = tl;
                    fits += 1;
                }
                out(fits);
            }
        }
        out(rh); out(rl);
    }
}
