//! Square roots, for the swarm's shadows, the first step: a, the whole square root of a number's hundreds (a long),
//! by taking away 1, 3, 5, ... while they fit (their sum is a^2). Deck: the count, then each number's hundreds and
//! the rest (0..99), as sw_hund punches them. Punches for sw_isqrt2: what is left of the hundreds (under 2 a + 1)
//! times 100 plus the rest, and 20 a + 1 (longs), then a's hundreds and the rest of a times ten.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut xh = inp();
        let mut xl = inp();
        let mut a = 0;
        let mut odd = 1;
        loop {
            let mut nh = xh;
            let mut nl = xl - odd;
            if nl < 0 { nl += 999; nl += 1; nh -= 1; }
            if nh < 0 { break; }
            xh = nh; xl = nl;
            a += 1;
            odd += 2;
        }
        // what is left times 100 plus the last two digits
        let lh = digit(xl, 0);
        out(lh * 10 + digit(xl, 1));
        out(digit(xl, 2) * 100 + inp());
        // 20 a + 1: 2 a's hundreds are its thousands
        let a2 = a + a;
        let th = digit(a2, 0);
        out(th); out((a2 - th * 100) * 10 + 1);
        let hundreds = digit(a, 0);
        out(hundreds); out((a - hundreds * 100) * 10);
    }
}
