//! Glide, twice a frame of the Mac's desk while the pointer moves, at step j of n of a glide from one point to another,
//! easing out (1 - (1 - j / n)^3 of the way): how much of a way is still to go, v (m / n)^3, m = n - j (v times m / n
//! three times over, each toward zero). Deck: j, n (1 <= j <= n <= 9), how many ways, then each: where it starts and
//! where it ends. Punches what is left of each. Run once on the glide's way across and down (start to end), then on
//! what is left of those (from 0), for mac_arc.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let step = inp();
    let steps = inp();
    let m = steps - step;
    let count = inp();
    for _ in 0..count {
        let from = inp();
        let way = inp() - from;
        // |way| m / n three times, without the products, which would pass a card: added m times, n taken away
        // whenever the sum reaches it, counting
        let mut size = abs(way);
        for _ in 0..3 {
            let mut sum = 0;
            let mut quotient = 0;
            for _ in 0..m {
                sum += size;
                while sum >= steps {
                    sum -= steps;
                    quotient += 1;
                }
            }
            size = quotient;
        }
        if way < 0 { out(-size); } else { out(size); }
    }
}
