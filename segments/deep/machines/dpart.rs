//! Part, per frame of the dive for the jellyfish (their bells' rings, points and facing). Deck: how many more
//! tenths to take (0, 1 or 2: v t / 100, / 1000 or / 10000), how many, then pairs v, t within 999 either way.
//! Punches, per pair, v t / 100 by v's digits 100 a + 10 b + c, ((c t / 10 + b t) / 10 + a t), with v's sign, then
//! a tenth of it as many times as asked: no product passes a card.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let tenths = inp();
    let count = inp();
    for _ in 0..count {
        let v = inp();
        let t = inp();
        let size = abs(v);
        let mut part = 0;
        for k in 0..3 {
            for _ in 0..digit(size, 2 - k) { part += t; }
            if k < 2 { part /= 10; }
        }
        if v < 0 { part = -part; }
        for _ in 0..tenths { part /= 10; }
        out(part);
    }
}
