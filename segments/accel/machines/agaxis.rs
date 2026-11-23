//! Glow axes, twice per light per frame of the accelerated drive: how far out its glow's profile is at each pixel
//! along one way from its middle. Deck: the step k (4 c across, 128 down: 32 a radius), the radius r (quarters of a
//! pixel), the first pixel out wanted and the one past the last. Punches for each pixel out d from the first, while
//! k d / r is at most 192 (six radii), t = k d / r rounded down and 3 t / 8 rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let k = inp();
    let r = inp();
    let from = inp();
    let to = inp();
    let mut t = 0;
    let mut rest = 0;
    let mut e = 0;
    let mut e8 = 0;
    let mut d = 0;
    while d < to {
        if d > 0 {
            rest += k;
            while rest >= r {
                rest -= r;
                t += 1;
                e8 += 3;
                if e8 >= 8 { e8 -= 8; e += 1; }
            }
        }
        if t > 192 || (t == 192 && rest > 0) { break; }
        if d >= from { out(t); out(e); }
        d += 1;
    }
}
