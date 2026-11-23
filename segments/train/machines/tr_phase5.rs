//! Patterns, for the train: where a place falls in a pattern repeating every 100000 / r, from t = u r given as its two
//! parts (the place's thousands times r, and its rest times r: longs). Deck: the count, then the parts. Punches the
//! pattern's cell md(fl(t, 100000), 1000) and how far into it, thousandths: fl(md(t, 100000), 100).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        // t = p 1000 + q as three cards: m, th, r (th and r 0..999)
        let mut m = inp();
        let mut th = inp();
        let q_hi = inp();
        let r = inp();
        let over = th - (999 - q_hi);
        if over > 0 { th = over - 1; m += 1; } else { th = over + 999; }
        let mut md = m % 100;
        if md < 0 { md += 100; }
        out(md * 10 + th / 100);
        out((th % 100) * 10 + r / 100);
    }
}
