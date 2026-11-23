//! Tall edges, for the swarm: whether a polygon's edge is 1000 pixels tall or more (its ends' places down, in
//! hundredths of a pixel, longs), to be cut at its middle. Deck: the count of edges; then each: its ends' places
//! down. Punches 1 for each tall edge, else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let (ah, al) = (inp(), inp());
        let (mut dh, mut dl) = (inp(), inp());
        // d = c - a, borrowing
        dh -= ah;
        dl -= al;
        if dl < 0 { dl += 999; dl += 1; dh -= 1; }
        // 100000 or more either way: d's thousands 100 or more, or below 0 and under -100 (or -100 and nothing more)
        if dh > 99 || dh < -100 || (dh == -100 && dl == 0) { out(1); } else { out(0); }
    }
}
