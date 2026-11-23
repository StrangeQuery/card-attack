//! Digits, once a frame of a UI layer, for each number it shows: the number written out. Deck: how many numbers;
//! then each (0-999). Punches for each: how many digits, then each one's character (48 for 0, ... 57 for 9), with no
//! leading zeros.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let value = inp();
        let hundreds = digit(value, 0);
        let tens = digit(value, 1);
        let units = digit(value, 2);
        if hundreds > 0 {
            out(3);
            out(hundreds + 48);
            out(tens + 48);
        } else if tens > 0 {
            out(2);
            out(tens + 48);
        } else {
            out(1);
        }
        out(units + 48);
    }
}
