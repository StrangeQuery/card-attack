//! Street dashes, once per row of the street per frame of the accelerated drive: each pixel's place in the dash's 30
//! quarters (16ths, 0-479) between the 25 sample columns (every 32 pixels), the nearer way round. Deck: how many
//! pixels a span (32; the software half's 8, from asg4's spans), then for each of the 24 spans (asgap's): the first sample's place p, the way to the next's (-1 or 1), and |g| (2 i + 1) / 64 to
//! start (whole, 64ths) and a pixel (whole, 64ths). Punches for each pixel p + g (2 i + 1) / 64 round
//! 480, the step rounded toward p.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let n = inp();
    for _ in 0..24 {
        let mut p = inp();
        let back = inp() < 0;
        let mut whole = inp();
        let mut part = inp();
        let step = inp();
        let half = inp();
        if back { p += 480; }
        for _ in 0..n {
            let mut x = if back { p - whole } else { p + whole };
            if x >= 480 { x -= 480; }
            out(x);
            whole += step;
            part += half;
            if part >= 64 { part -= 64; whole += 1; }
        }
    }
}
