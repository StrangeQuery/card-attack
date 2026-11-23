//! In tenths, per frame of the dive while the portal shows. Deck: how many, then places as a whole part and
//! hundredths. Punches, per place, it in tenths, rounded down; 400 when it is further than 30 units either way.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let whole = inp();
        let part = inp();
        if abs(whole) > 30 { out(400); } else { out(whole * 10 + part / 10); }
    }
}
