//! Hold, a few times per frame of the Rubik's cube. Deck: groups, each the number of values n (0 ends the deck),
//! the least and the most, then the n values. Punches each value held between the least and the most.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let n = inp();
        if n <= 0 { break; }
        let least = inp();
        let most = inp();
        for _ in 0..n {
            let mut value = inp();
            if value < least { value = least; }
            if value > most { value = most; }
            out(value);
        }
    }
}
