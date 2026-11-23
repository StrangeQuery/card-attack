//! Flare sizes, once per flared light per frame of the accelerated drive. Deck: how many, then each: its strength
//! (500ths) and m (5 for headlights, 3 for a lamp). Punches each phi = min(600, s) 7 / 5 m / 5 (thousandths, rounded
//! down each time), r = phi / 25 (pixels) and dg = 90 + phi / 20.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut s = inp();
        if s > 600 { s = 600; }
        let m = inp();
        let mut a = 0;
        let mut sum = 0;
        for _ in 0..s { sum += 7; while sum >= 5 { sum -= 5; a += 1; } }
        let mut phi = 0;
        sum = 0;
        for _ in 0..a { sum += m; while sum >= 5 { sum -= 5; phi += 1; } }
        out(phi);
        out(phi / 25);
        out(90 + phi / 20);
    }
}
