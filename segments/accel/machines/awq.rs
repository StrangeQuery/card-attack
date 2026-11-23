//! Wall size, once per wall a column's ray meets per frame of the accelerated drive: how fast the wall's texels pass
//! the rows there. Deck: how many, then each: the ray (across, along), the face's distance across (eighths), the
//! block's start (quarters ahead), 1 for an end wall (0 a face). Punches each Q = q / d, twentieths of a texel a
//! half row (10 s / along for an end wall, s the start at least 16; 5 |e| / |across| for a face), as its whole and
//! what is left, and d.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let across = abs(inp());
        let along = inp();
        let e = abs(inp());
        let mut s = inp();
        let end = inp() > 0;
        if s < 16 { s = 16; }
        // n m / d by adding: m added n times, d taken away whenever the sum reaches it
        let (n, m, d) = if end { (10, s, along) } else { (5, e, across) };
        let mut q = 0;
        let mut sum = -d;
        for _ in 0..n {
            sum += m;
            while sum >= 0 { sum -= d; q += 1; }
        }
        out(q);
        out(sum + d);
        out(d);
    }
}
