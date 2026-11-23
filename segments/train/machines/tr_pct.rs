//! Shares, for the train: a t / 100 cut toward zero (a a card, t -100..100). Deck: the count, then pairs a, t.
//! Punches each.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let mut rest = a;
        let mut add = inp();
        // t times the digits of |rest|, units first, the part so far cut to a tenth before each (each part has the
        // product's sign, so cutting each toward zero cuts the whole the same; nothing on the way passes 989); t
        // times a digit: 4 t while it fits, then 2 t, then t (a t of 100 too: a part reaches 999 at most).
        if rest < 0 { add = -add; rest = -rest; }
        let mut part = 0;
        let twice = add + add;
        for _digit in 0..3 {
            part /= 10;
            let mut d = rest % 10;
            rest /= 10;
            loop {
                let e = d - 4;
                if e < 0 { break; }
                d = e;
                part = part + twice + twice;
            }
            let e = d - 2;
            if e >= 0 { d = e; part += twice; }
            if d >= 1 { part += add; }
        }
        out(part);
    }
}
