//! Angles, for the swarm's sines: an angle in millionths of a turn (a long, any whole turns) as the places in the
//! printed table of 999 sin every thousandth of a turn (a quarter turn of it, each entry printed with the next) for its
//! sine and its cosine. Deck: the count, then the angles. Punches for each, for the sine and then the cosine (a
//! quarter turn on): the table's entry below (0..250), how far on in tenths (0..9), and 1 for the second half turn
//! (else 0).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut hi = inp();
        let lo = inp();
        // SAFETY: a long's rest is 0..999
        unsafe { assume(lo >= 0 && lo <= 999); }
        if hi < 0 { hi += 999; hi += 1; }
        let tenths = digit(lo, 0);
        for _ in 0..2 {
            // whole thousandths into the turn, its quarter, and the place in that quarter
            let mut quarter = 0;
            let mut m = hi;
            while m >= 250 { m -= 250; quarter += 1; }
            let (mut at, mut part) = (m, tenths);
            if quarter == 1 || quarter == 3 {
                // the second and fourth quarters: the first's mirror
                at = 250 - m;
                if part > 0 { at -= 1; part = 10 - part; }
            }
            out(at);
            out(part);
            if quarter >= 2 { out(1); } else { out(0); }
            // a quarter turn on: past 999 just when hi is more than 749
            hi -= 749;
            if hi > 0 { hi -= 1; } else { hi += 999; }
        }
    }
}
