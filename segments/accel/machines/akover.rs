//! Covers, as many as the host has, per frame of the accelerated drive: how much a layer covers, (a - b) c / 32 (a,
//! b, c in 32nds), rounded down (a - b itself for c = 32). Deck: how many, then each a, b, c. Punches each.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp() - inp();
        let c = inp();
        // a added c times, 32 taken away whenever the sum reaches it (both at most 32: the product passes a card)
        let mut k = 0;
        let mut sum = -32;
        if c == 32 { k = a; }
        else { for _ in 0..c {
            sum += a;
            while sum >= 0 { sum -= 32; k += 1; }
        } }
        out(k);
    }
}
