//! Walks, for the mesh kit: a value carried down an edge's rows or across a span's pixels. Deck: the count; 1 to
//! punch each place (else 0); then each walk: its first place and the place after its last (at most 998 on), and the
//! values at its two ends (at most 999 apart). Punches for each of its n places (the place, when asked, and) the
//! first end's value plus the change to the other's times (k + 1) / (n + 1) at the k-th, cut toward zero: the change
//! divided once, then its remainder carried a place at a time.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let places = inp();
    for _ in 0..count {
        let mut at = inp();
        let n = inp() - at;
        let mut value = inp();
        // the change's size over n + 1, by taking it away, counting toward the change's sign
        let steps = n + 1;
        let mut size = inp() - value;
        let mut one = 1;
        if size < 0 { size = -size; one = -1; }
        let mut whole = 0;
        while size >= steps { size -= steps; whole += one; }
        // the remainder so far less n + 1: below 0 until it carries one
        let mut over = -steps;
        for _ in 0..n {
            value += whole;
            over += size;
            if over >= 0 { over -= steps; value += one; }
            if places > 0 { out(at); at += 1; }
            out(value);
        }
    }
}
