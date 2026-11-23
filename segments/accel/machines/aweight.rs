//! Eighths, as many as the host has, per frame of the accelerated drive: a place along in quarters as entries of the
//! plan (8 quarters each) and quarters into one. Deck: how many, then each two numbers to add (quarters, the sum 0 or
//! more). Punches each the sum / 8 and the sum mod 8.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut rest = inp() + inp();
        let mut entries = 0;
        while rest >= 80 { rest -= 80; entries += 10; }
        while rest >= 8 { rest -= 8; entries += 1; }
        out(entries);
        out(rest);
    }
}
