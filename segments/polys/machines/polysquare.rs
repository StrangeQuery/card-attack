//! Square, once per frame of the polys: the size of each corner's normal, squared. Deck: the count, then each
//! normal's three parts (in hundredths, -199..199). Punches the sum of their squares as hundreds and the rest
//! (0..99). A part is 100 a + b (a 0 or 1); its square is 10000 a + 200 a b and b b, which is 10 b t + b u for b's
//! tens and units t and u: b added t times, and b added u times to the rest so far, so nothing passes 990.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut hundreds = 0;
        let mut rest = 0;
        for _part in 0..3 {
            let mut b = inp();
            if b < 0 { b = -b; }
            if b >= 100 { b -= 100; hundreds += 100 + 2 * b; }
            let tens = b / 10;
            let units = b % 10;
            let mut by_tens = 0;
            for _ in 0..tens { by_tens += b; }
            let mut sum = rest;
            for _ in 0..units { sum += b; }
            hundreds += by_tens / 10 + sum / 100;
            rest = sum % 100 + by_tens % 10 * 10;
            if rest >= 100 { rest -= 100; hundreds += 1; }
        }
        out(hundreds);
        out(rest);
    }
}
