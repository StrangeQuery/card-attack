//! Square roots, for the swarm's shadows, the second step: the root's last digit b, the most for which (20 a + b) b
//! fits in what was left of the hundreds times 100 plus the number's last two digits, by taking away 20 a + 1,
//! 20 a + 3, ... while they fit. Deck: the count, then sw_isqrt's cards for each. Punches each root, 10 a + b (a long).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut rh = inp();
        let mut rl = inp();
        let mut th = inp();
        let mut tl = inp();
        let mut b = 0;
        loop {
            let mut nh = rh - th;
            let mut nl = rl - tl;
            if nl < 0 { nl += 999; nl += 1; nh -= 1; }
            if nh < 0 { break; }
            rh = nh; rl = nl;
            b += 1;
            // two more: past 999 just when tl is more than 997
            tl -= 997;
            if tl > 0 { tl -= 1; th += 1; } else { tl += 999; }
        }
        out(inp()); out(inp() + b);
    }
}
