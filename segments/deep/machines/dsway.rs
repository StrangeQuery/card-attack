//! Sway, per frame of the dive for the kelp. Deck: how many points up the stalks, then for each its stalk's sway
//! phase (a step 0..59 of a 60-step turn) and its height up the stalk (tenths). The sway runs up the stalk, the top
//! lagging: punches, per point, the step there, phase - 0.33 height (radians 0.35 a unit), round the 60.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let phase = inp();
        let height = inp();
        // 33 height / 100 by height's digits
        let mut part = 0;
        for k in 0..3 {
            for _ in 0..digit(height, 2 - k) { part += 33; }
            if k < 2 { part /= 10; }
        }
        let mut step = phase + 120 - part;
        while step >= 60 { step -= 60; }
        out(step);
    }
}
