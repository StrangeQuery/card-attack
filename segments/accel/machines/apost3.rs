//! Post arm columns, once per lamp post drawn per frame of the accelerated drive. Deck: how many, then each: the
//! post's pixel across and its head's. Punches each the arm's columns, from the lesser pixel less 1 (at least 0) to
//! the greater and 2 (at most 768).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let x = inp();
        let head = inp();
        let mut lo = (if head < x { head } else { x }) - 1;
        let mut hi = (if head < x { x } else { head }) + 2;
        if lo < 0 { lo = 0; }
        if hi > 768 { hi = 768; }
        out(lo);
        out(hi);
    }
}
