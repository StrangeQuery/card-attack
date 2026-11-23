//! Wall texel rows, as many as the host has, per frame of the accelerated drive, a channel at a time: the two texel
//! columns either side of where the ray meets the wall, mixed. Deck: how many, then each: the near texel's channel,
//! the next one's (tenths of a step, hazed), and tenths of the way (0-9). Punches each a + (b - a) t / 10, the step
//! rounded toward a: |b - a| = 10 q + r, so it is q t + r t / 10 (the product would pass a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let t = inp();
        let size = abs(b - a);
        let tens = size / 10;
        let ones = size - 10 * tens;
        let mut step = 0;
        for _ in 0..t { step += tens; }
        let mut part = 0;
        for _ in 0..t { part += ones; }
        step += part / 10;
        out(if b < a { a - step } else { a + step });
    }
}
