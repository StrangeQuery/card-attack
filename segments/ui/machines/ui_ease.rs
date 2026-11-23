//! Ease, once a frame of a UI layer, for every number that moves in it (and once for each shadow's ramps): from one
//! number toward another, step j of n. Deck: how many; then each: 3 smooth or 1 straight, from, to, j, n (n >= 1,
//! 0 <= j <= n, |to - from| + n at most 999). Punches each: from + (to - from) u, u = j / n, or for smooth
//! from + (to - from) u u (3 - 2 u); each product rounded down, so it moves from `from` toward `to`.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let passes = inp();
        let from = inp();
        let to = inp();
        let step = inp();
        let steps = inp();
        // u times the difference; for smooth, u times that, then (3 - 2 u) times that: each product without the
        // product, which would pass a card: the number added `times` times, n taken away whenever the sum reaches
        // it, counting
        let mut part = abs_diff(to, from);
        let mut times = step;
        for pass in 0..passes {
            if pass == 2 { times = steps * 3 - step - step; }
            let mut sum = 0;
            let mut quotient = 0;
            for _ in 0..times {
                sum += part;
                while sum >= steps {
                    sum -= steps;
                    quotient += 1;
                }
            }
            part = quotient;
        }
        if to < from { out(from - part); } else { out(from + part); }
    }
}
