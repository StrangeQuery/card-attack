//! Sixteenths, as many as the host has, per frame of the accelerated drive: some times a number of 16ths and a few
//! more, as pixels and 16ths (a sum that would pass a card). Deck: how many, then each: the number (0 or more), how
//! many times, the 16ths more. Punches each the sum's pixels and 16ths (0-15).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let v = inp();
        let times = inp();
        let mut f = inp();
        let mut p = 0;
        for _ in 0..times {
            f += v;
            while f >= 16 { f -= 16; p += 1; }
        }
        out(p);
        out(f);
    }
}
