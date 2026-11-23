//! Places in tables that repeat, for the train: md(fl(u, 10), 1000) of a long u (a place over 10, its last three
//! digits). Deck: the count, then the longs. Punches each.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let hi = inp();
        let lo = inp();
        // (hi 1000 + lo) / 10 md 1000: hi's last digit (0..9) times 100 and lo / 10
        let mut d = hi % 10;
        if d < 0 { d += 10; }
        out(d * 100 + lo / 10);
    }
}
