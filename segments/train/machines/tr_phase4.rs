//! Patterns, for the train: where a place falls in a pattern repeating every 10000 / r, from t = u r given as its two
//! parts (the place's thousands times r, and its rest times r: longs). Deck: the count, then the parts. Punches the
//! pattern's cell md(fl(t, 10000), 1000) and how far into it, thousandths: fl(md(t, 10000), 10).
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
        let mut md = m % 10;
        if md < 0 { md += 10; }
        out(md * 100 + th / 10);
        out((th % 10) * 100 + r / 10);
    }
}
