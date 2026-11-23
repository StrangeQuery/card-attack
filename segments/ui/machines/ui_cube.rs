//! Cube, once a frame of a UI layer, for every number that moves by an ease out (a pointer's glide): from one number
//! toward another, step j of n, fast then slowing. Deck: how many; then each: from, to, j, n (n >= 1, 0 <= j <= n,
//! |to - from| + n at most 999). Punches each: to - (to - from) w w w, w = (n - j) / n, each product rounded down (so
//! the glide never passes `to`): 1 - (1 - u) cubed of the way.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let from = inp();
        let to = inp();
        let step = inp();
        let steps = inp();
        let times = steps - step;
        // the difference times w three times over, each product by adding: the difference added `times` times, n
        // taken away whenever the sum reaches it, counting (a product would pass a card)
        let mut part = abs_diff(to, from);
        for _ in 0..3 {
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
        if to < from { out(to + part); } else { out(to - part); }
    }
}
