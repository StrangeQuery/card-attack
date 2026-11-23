//! Root, once per pixel for the whole dive. Deck: a number as thousands and units (up to 999,999). Punches half its
//! square root, rounded down: four times each odd number 1, 3, 5, ... taken away while it fits, counting (the
//! first n of them add up to 4 n^2).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut thousands = inp();
    let mut units = inp();
    let mut odd = 1;
    let mut root = 0;
    loop {
        // take 4 odd away: when that goes below nothing, it didn't fit
        for _ in 0..4 {
            units -= odd;
            if units < 0 { units += 500; units += 500; thousands -= 1; }
        }
        if thousands < 0 { break; }
        odd += 2;
        root += 1;
    }
    out(root);
}
