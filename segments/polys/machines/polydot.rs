//! Dot, once per list per frame of the polys: sums of products, for the blob's corners. Deck: the count, the terms
//! to an item and a bias; then for each item an addend and that many pairs (c, v), -99 <= c <= 99. Punches for each
//! item the bias plus the addend plus each c v / 100 (each cut toward zero).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let terms = inp();
    let bias = inp();
    for _ in 0..count {
        let mut sum = bias + inp();
        let mut left = terms;
        while left > 0 {
            left -= 1;
            // c v / 100, toward zero: c times v's digits, units first, the part so far cut to a tenth before each:
            // ((c u / 10 + c t) / 10) + c h. Every part has the product's sign, so cutting each toward zero cuts the
            // whole the same, and nothing on the way passes 989. c times a digit: 4 c while it fits, then 2 c, then c.
            let mut add = inp();
            let mut rest = inp();
            if rest < 0 { add = -add; rest = -rest; }
            let twice = add + add;
            let mut part = 0;
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
            sum += part;
        }
        out(sum);
    }
}
