//! Root, once per frame of the polys: the size of each corner's normal. Deck: the count, then each normal's size
//! squared as hundreds and the rest. Punches its square root, cut down: the odd numbers 1, 3, 5 ... that still fit
//! taken away (their sums are the squares), ten at a time while a hundred-odd block fits in the hundreds, then
//! one at a time.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut hundreds = inp();
        let mut rest = inp();
        // ten odd numbers from 2 root + 1 add to 20 root + 100: in hundreds, root / 5 + 1, for root a multiple of 10
        let mut root = 0;
        let mut block = 1;
        while hundreds >= block {
            hundreds -= block;
            block += 2;
            root += 10;
        }
        let mut odd = 2 * root + 1;
        loop {
            rest -= odd;
            while rest < 0 { rest += 100; hundreds -= 1; }
            if hundreds < 0 { break; }
            odd += 2;
            root += 1;
        }
        out(root);
    }
}
