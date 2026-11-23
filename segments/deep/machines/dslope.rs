//! Slopes, twice per screen row per frame of the dive (across, then up), once more for the water's pixels (up, over
//! the ray's length), and to put the things drawn over the seabed on the screen (across, or up, over ahead). Deck:
//! how many decimal places (2 or 3), how many pairs, then for each n and d, d 1 to 999 and n less than 10 d either
//! way (less than d for 3 places). Each ray goes forward one unit at a time and moves n / d per unit. Punches, per
//! pair, 100 n / d (or 1000 n / d) toward zero, worked out by long division.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let places = inp();
    let count = inp();
    for _ in 0..count {
        let n = inp();
        let d = inp();
        // long division: three digits, each counting how many times d can be taken away from what is left. Ten
        // times what is left can be up to 9,980, so it is kept as five hundreds and the rest.
        let mut fives = 0;
        let mut rest = abs(n);
        let mut quotient = 0;
        for _ in 0..places + 1 {
            let mut digit = 0;
            loop {
                // take d away from 500 fives + rest, if it goes: borrowing fives into the rest
                let mut left = rest - d;
                let mut f = fives;
                while left < 0 && f > 0 { left += 500; f -= 1; }
                if left < 0 { break; }
                rest = left;
                fives = f;
                digit += 1;
            }
            quotient = quotient * 10 + digit;
            // ten times what is left (under d): 10 fives (and the rest's own fifties) and ten times the rest
            fives *= 10;
            while rest >= 50 { rest -= 50; fives += 1; }
            rest *= 10;
        }
        if n < 0 { quotient = -quotient; }
        out(quotient);
    }
}
