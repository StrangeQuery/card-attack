//! Ring distance, once per tile row of the plasma at 192x96 (and of the ray's plasma floor). Deck: 96 points (u, v)
//! on the plasma's plane. Punches each point's distance from the rings' centre (0, 48), rounded to the nearest
//! unit, halves up (the r with r^2 - r <= n < r^2 + r, so 1 at the centre itself). A square root of up to 491 squared can't be taken on cards, so the distance is walked: from the centre,
//! one unit at a time to each point in turn, keeping r and how far r^2 is off, e = n - r^2 for n = u^2 + w^2
//! (w = v - 48), in -r <= e < r. A step from c to c + 1 changes n by c + (c + 1), and from c to c - 1 by
//! -c + (-c + 1): each added in two halves, so that e stays inside a card, and whenever e leaves its range r moves
//! by one. A row's points are close together, so the walk is short.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut u = 0;
    let mut w = 0;                     // v - 48
    let mut r = 1;                     // at the centre n = 0: r = 1, e = -1
    let mut e = -1;
    for _ in 0..96 {
        let to_u = inp();
        let to_w = inp() - 48;
        loop {
            let mut half;              // the first half of the change in n
            if u < to_u {
                half = u;
                u += 1;
            } else if u > to_u {
                half = -u;
                u -= 1;
            } else if w < to_w {
                half = w;
                w += 1;
            } else if w > to_w {
                half = -w;
                w -= 1;
            } else {
                break;
            }
            for _ in 0..2 {
                e += half;
                while e >= r {          // (r + 1)^2 = r^2 + r + (r + 1)
                    e -= r;
                    r += 1;
                    e -= r;
                }
                while e + r < 0 {       // e < -r: (r - 1)^2 = r^2 - r - (r - 1)
                    e += r;
                    r -= 1;
                    e += r;
                }
                half += 1;             // the second half
            }
        }
        out(r);
    }
}
