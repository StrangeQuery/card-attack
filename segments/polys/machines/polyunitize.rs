//! Unitize, once per frame of the polys: a normal's part made a unit's. Deck: the count, then for each part the
//! normal's size and the part (-199..199, in hundredths; no more than the size). Punches 100 part / size, cut toward
//! zero: long division, the remainder taken five, five, two and two times (so it never passes 5 x 199).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");
use demokit::mul::times;

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let size = inp();
        let part = inp();
        let mut left = part;
        if part < 0 { left = -part; }
        let mut unit = 0;
        let mut by = 5;
        for step in 0..4 {
            left = times(left, by);
            unit = times(unit, by);
            while left >= size { left -= size; unit += 1; }
            if step == 1 { by = 2; }
        }
        if part < 0 { unit = -unit; }
        out(unit);
    }
}
