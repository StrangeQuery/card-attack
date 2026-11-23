//! Scale, once per screen row per frame of the dive while the portal shows. Deck: how many, then pairs of a size M
//! (0..999) and a share t (200ths). Punches, per pair, M + M t / 200, the product taken by M's digits,
//! 100 a + 10 b + c: ((c t / 10 + b t) / 10 + a t) / 2.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let size = inp();
        let share = inp();
        let mut part = 0;
        for k in 0..3 {
            for _ in 0..digit(size, 2 - k) { part += share; }
            if k < 2 { part /= 10; }
        }
        out(size + part / 2);
    }
}
