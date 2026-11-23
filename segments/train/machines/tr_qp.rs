//! Products from quarter squares, for the train: the difference of the two the host looked up (longs), then over
//! 10^s rounded down (s 0..6). Deck: the count, s; then each product's two quarter squares (longs). Punches each as
//! a long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let s = inp();
    for _ in 0..count {
        let mut hi = inp();
        let mut lo = inp();
        hi -= inp();
        lo -= inp();
        if lo < 0 { lo += 999; lo += 1; hi -= 1; }
        // over 10^s: whole cards moved down for each 3, digits for the rest (rounded down, the rest never below 0)
        let mut k = s;
        while k >= 3 {
            k -= 3;
            lo = if hi < 0 { hi + 999 + 1 } else { hi };
            hi = if hi < 0 { -1 } else { 0 };
        }
        while k > 0 {
            k -= 1;
            // (hi 1000 + lo) / 10: hi / 10 thousands, and (hi % 10) 100 + lo / 10 the rest
            let mut q = hi / 10;
            let mut r = hi - q * 10;
            if r < 0 { r += 10; q -= 1; }
            hi = q;
            lo = r * 100 + lo / 10;
        }
        out(hi);
        out(lo);
    }
}
