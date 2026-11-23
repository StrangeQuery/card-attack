//! Walks by slopes, for the train's edges and spans: a value carried across n places (n a long), each step the
//! slope's whole part (a long) and one more each time its thousandths carry (from f0), so the k-th place has the
//! start's value + fl(f0 + a k, 1000). Deck: the count; then each walk: n (a long); the start's value (a long); the
//! slope's whole part (a long) and its thousandths; f0. Punches each place's value, a long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut c_hi = inp();
        let mut c_lo = inp();
        let mut v_hi = inp();
        let mut v_lo = inp();
        let w_hi = inp();
        let w_lo = inp();
        let frac = inp();
        let mut acc = inp();
        loop {
            if c_lo == 0 {
                if c_hi == 0 { break; }
                c_hi -= 1;
                c_lo = 999;
            } else { c_lo -= 1; }
            out(v_hi);
            out(v_lo);
            v_hi += w_hi;
            let over = v_lo - (999 - w_lo);
            if over > 0 { v_lo = over - 1; v_hi += 1; } else { v_lo = over + 999; }
            let over = acc - (999 - frac);
            if over > 0 {
                acc = over - 1;
                if v_lo == 999 { v_lo = 0; v_hi += 1; } else { v_lo += 1; }
            } else { acc = over + 999; }
        }
    }
}
