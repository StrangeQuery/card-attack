//! Edge rows on the screen, for the swarm's polygons: where an edge's rows leave the bottom of the screen. Deck: the
//! count, then each edge's rows, how many are above the screen, and the screen row of its first on it. Punches for
//! each the row (counted from its first) after its last on the screen.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut end = inp();
        let above = inp();
        let row = inp();
        // the screen's last row is 383
        if end - above > 384 - row { end = 384 - row + above; }
        // none on the screen when its first would be below it
        if end < above { end = above; }
        out(end);
    }
}
