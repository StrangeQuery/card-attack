//! Times, once per sample column per frame of the accelerated drive (three times): n m / a for each step of the
//! march, rounded down, and what it leaves. Deck: how many, then each: n as b - c (signed), m and a (1-999). Punches each
//! the quotient (about 950 at most either way: it stops counting there) and what is left (0 to a - 1).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let n = inp() - inp();
        let m = inp();
        let a = inp();
        // the smaller of |n| and m counts the additions of the other; a taken away whenever the sum reaches it (the
        // sum kept below 0 by starting at -a, so it never passes a card)
        let mut times = abs(n);
        let mut each = m;
        if m < times { times = m; each = abs(n); }
        let mut q = 0;
        let mut sum = -a;
        while times > 0 && q < 950 {
            sum += each;
            while sum >= 0 { sum -= a; q += 1; }
            times -= 1;
        }
        sum += a;
        // rounded down below zero: one more, and what is left from the other side
        if n < 0 { q = -q; if sum > 0 { q -= 1; sum = a - sum; } }
        out(q);
        out(sum);
    }
}
