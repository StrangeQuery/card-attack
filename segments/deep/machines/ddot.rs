//! Dot, per frame of the dive for the things drawn over the seabed (snow, jellyfish, kelp, fish). Deck: one of the
//! camera's vectors (forward, right or up: x, y, z in hundredths of the unit vector), how many points, then for each
//! its place from the camera (x, y, z in tenths of a unit, within 999). Punches, per point, how far it is along the
//! vector, in tenths. Each product is split by the place's digits so that none passes a card: for a place
//! 100 a + 10 b + c, ((c v / 10 + b v) / 10 + a v), toward zero.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut vector = [0i16; 3];
    for c in 0..3 { vector[c] = inp(); }
    let count = inp();
    for _ in 0..count {
        let mut along = 0;
        for c in 0..3 {
            let place = inp();
            let size = abs(place);
            let mut part = 0;
            for k in 0..3 {
                for _ in 0..digit(size, 2 - k) { part += vector[c]; }
                if k < 2 { part /= 10; }
            }
            if place < 0 { part = -part; }
            along += part;
        }
        out(along);
    }
}
