//! Ghost places, per ghost per frame of the accelerated drive: how far t of the way from a light's pixel to the
//! screen's middle is. Deck: how many, then each: the light's pixel p, the middle's m, t (hundredths). Punches each
//! |m - p| t / 100 rounded down (or 996 once past 995), 1 if anything was left over, else 0, and the way (1 when the
//! middle is at or past p, else -1).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let p = inp();
        let m = inp();
        let t = inp();
        let mut k = if m < p { p - m } else { m - p };
        let mut q = 0;
        let mut rest = 0;
        while k > 0 && q <= 995 {
            rest += t;
            while rest >= 100 { rest -= 100; q += 1; }
            k -= 1;
        }
        out(if q > 995 { 996 } else { q });
        out(if rest > 0 { 1 } else { 0 });
        out(if m < p { -1 } else { 1 });
    }
}
