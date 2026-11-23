//! Mixes, for the train, the first half: a + (b - a) t / 100 needs the product (b - a) t, by quarter squares as
//! tr_qa. Deck: the count, then a, b, t (cards; b - a a card). Punches the two sizes to look up, as tr_qa.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let d = inp() - a;
        let t = inp();
        let x = abs(d);
        let y = abs(t);
        let over = x - (999 - y);
        let small = abs(x - y);
        if (d < 0) == (t < 0) {
            if over > 0 { out(1); out(over - 1); } else { out(0); out(over + 999); }
            out(0); out(small);
        } else {
            out(0); out(small);
            if over > 0 { out(1); out(over - 1); } else { out(0); out(over + 999); }
        }
    }
}
