//! Dot, many times per frame of the Rubik's cube: sums of products. Deck: groups, each the number of products n (0
//! ends the deck), then n pairs x, y (-999..999). Each product is x y / 999, its size rounded half up. Punches each group's sum, then its negative.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let n = inp();
        if n <= 0 { break; }
        let mut sum = 0;
        for _ in 0..n {
            let mut count = inp();
            let mut add = inp();
            if count < 0 { count = -count; add = -add; }
            let negative = add < 0;
            if negative { add = -add; }
            // count add / 999 without the product, which would pass a card: add taken count times, carrying a
            // 999 whenever the rest reaches it, the rest starting at 499 to round
            let gap = 999 - add;        // the rest at or past this carries
            let mut rest = 499;
            let mut product = 0;
            for _ in 0..count {
                if rest >= gap { rest -= gap; product += 1; } else { rest += add; }
            }
            if negative { sum -= product; } else { sum += product; }
        }
        out(sum);
        out(-sum);
    }
}
