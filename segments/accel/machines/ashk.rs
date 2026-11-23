//! Shadow covers, once per frame of the accelerated drive while our car's shadow shows. Deck: its darkness k (32nds).
//! Punches for each e (u u + v v) from 0 to 319 k min(256, 320 - e) / 256, rounded down (32nds).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let k = inp();
    for e in 0..320 {
        let mut v = 320 - e;
        if v > 256 { v = 256; }
        let mut q = 0;
        let mut sum = 0;
        for _ in 0..k {
            sum += v;
            while sum >= 256 { sum -= 256; q += 1; }
        }
        out(q);
    }
}
