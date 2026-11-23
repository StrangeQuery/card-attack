//! Between, as many as the host has, per frame of the accelerated drive: a number that part of the way from one to
//! another. Deck: how many, then each: p, q, n of d (0 <= n <= d, d 1-999). Punches each p + (q - p) n / d, the step
//! rounded toward p, q - p kept within 999 either way.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let p = inp();
        let q = inp();
        let n = inp();
        let d = inp();
        // the smaller of |q - p| and n counts the additions of the other; d taken away whenever the sum reaches it
        // (kept below 0 by starting at -d)
        // q - p, kept within 999 (it would pass a card): the far side checked first, only where it can be past
        let mut gap = 999;
        if p > 0 && q < p - 999 { gap = -999; } else if p >= 0 || q <= p + 999 { gap = q - p; }
        let size = abs(gap);
        let mut times = size;
        let mut each = n;
        if n < size { times = n; each = size; }
        let mut step = 0;
        let mut sum = -d;
        while times > 0 {
            sum += each;
            while sum >= 0 { sum -= d; step += 1; }
            times -= 1;
        }
        if gap < 0 { step = -step; }
        out(p + step);
    }
}
